use std::any::Any;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::execution_engine::{ExecutionEngine, JitFunction};
use inkwell::module::{Linkage, Module};
use inkwell::{AddressSpace, FloatPredicate, OptimizationLevel};
use inkwell::basic_block::BasicBlock;
use inkwell::types::{StructType};
use inkwell::values::{AnyValue, BasicValue, FloatValue, FunctionValue};
use crate::{BinOp, Expr};
use crate::parser::{Program, Stmt};

#[no_mangle]
pub extern "C" fn print_float_64(num: f64) {
    println!("{}", num);
}

pub fn compile_and_run(prog: Program) {
    let context = Context::create();
    let mut codegen = CodeGen::new(&context);
    codegen.compile(prog, &context);
}

const ROOT: &'static str = "main";

type BodyFunc = unsafe extern "C" fn(f64) -> f64;

struct Scope<'ctx> {
    values: HashMap<String, FloatValue<'ctx>>,
}

struct Scopes<'ctx> {
    scopes: Vec<Scope<'ctx>>,
}

impl<'ctx> Scopes<'ctx> {
    pub fn new() -> Self {
        let scope = Scope { values: HashMap::default() };
        Scopes { scopes: vec![scope] }
    }
    pub fn insert(&mut self, name: String, value: FloatValue<'ctx>) {
        self.scopes.last_mut().unwrap().values.insert(name, value);
    }
    pub fn get(&self, name: &str) -> Option<&FloatValue<'ctx>> {
        for scope in self.scopes.iter().rev() {
            if let Some(val) = scope.values.get(name) {
                return Some(val);
            }
        }
        None
    }
    pub fn begin_scope(&mut self) {
        self.scopes.push(Scope { values: HashMap::default() });
    }
    pub fn end_scope(&mut self) -> HashMap<String, FloatValue<'ctx>> {
        let scope = self.scopes.pop().unwrap();
        scope.values
    }
    pub fn lookup(&self, name: &str) -> Option<FloatValue<'ctx>> {
        for scope in self.scopes.iter().rev() {
            if let Some(val) = scope.values.get(name) {
                return Some(val.clone());
            }
        }
        None
    }
}

struct CodeGen<'ctx> {
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    execution_engine: ExecutionEngine<'ctx>,
    counter: usize,
    value_type: StructType<'ctx>,
    scopes: Scopes<'ctx>,
    print_func: FunctionValue<'ctx>,
}

impl<'ctx> CodeGen<'ctx> {
    fn new(context: &'ctx Context) -> Self {
        let module = context.create_module("sum");
        let execution_engine = module.create_execution_engine().expect("To be able to create exec engine");
        let i8 = context.i8_type();
        let i64 = context.i64_type();
        let value_type = context.struct_type(&[i8.into(), i64.into()], false);
        let f64_type = context.f64_type();
        let print_f64_type = context.void_type().fn_type(&[f64_type.into()], false);
        let print_func = module.add_function("print_f64", print_f64_type, Some(Linkage::ExternalWeak));
        let codegen = CodeGen {
            module,
            builder: context.create_builder(),
            execution_engine,
            counter: 0,
            value_type,
            scopes: Scopes::new(),
            print_func,
        };
        codegen
    }
    fn name(&mut self) -> String {
        self.counter += 1;
        format!("tmp{}", self.counter)
    }
    fn compile(&mut self, prog: Program, context: &'ctx Context) {
        let f64_type = context.f64_type();

        let f64_func = f64_type.fn_type(&[], false);
        let function = self.module.add_function(ROOT, f64_func, Some(Linkage::External));

        let bb = context.append_basic_block(function, ROOT);

        self.builder.position_at_end(bb);

        for blk in prog.body {
            self.compile_stmt(blk.body, context);
        }

        let str = self.module.print_to_string().to_string().replace("\\n", "\n");

        self.module.write_bitcode_to_path(Path::new("/tmp/crawk.bc"));


        println!("{}", str);

        // unsafe { self.execution_engine.get_function(ROOT).ok() }.expect("to get root func")
    }

    fn compile_stmt(&mut self, stmt: Stmt, context: &'ctx Context) -> BasicBlock<'ctx> {
        match stmt {
            Stmt::Expr(_) => panic!("cannot compile expression stmt"),
            Stmt::Print(expr) => {
                let res = self.compile_expr(expr, context);
                self.builder.build_call(self.print_func, &[res.into()], "print_f64_call");
            }
            Stmt::Assign(name, expr) => {
                let fin = self.compile_expr(expr, context);
                // todo: check if name is already in scope SSA!
                self.scopes.insert(name.clone(), fin);
            }
            Stmt::Return(result) => {
                let fin = match result {
                    None => context.f64_type().const_float(0.0),
                    Some(val) => self.compile_expr(val, context),
                };
                self.builder.build_return(Some(&fin));
            }
            Stmt::Group(body) => {
                self.scopes.begin_scope();
                for stmt in body {
                    self.compile_stmt(stmt, context);
                }
                self.scopes.end_scope();
            }
            Stmt::If(test, true_blk, false_blk) => {
                let false_blk = if let Some(fal) = false_blk { fal } else { panic!("must have false block") };

                let then_bb = context.append_basic_block(self.module.get_function(ROOT).expect("root to exist"), "then");
                let else_bb = context.append_basic_block(self.module.get_function(ROOT).expect("root to exist"), "else");
                let continue_bb = context.append_basic_block(self.module.get_function(ROOT).expect("root to exist"), "merge");

                let predicate = self.compile_expr(test, context);
                let zero = context.f64_type().const_float(0.0);
                let comparison = self.builder.build_float_compare(FloatPredicate::OEQ, predicate, zero, "if-test");

                self.builder.build_conditional_branch(comparison, then_bb, else_bb);

                self.builder.position_at_end(then_bb);
                self.scopes.begin_scope();
                let then_bb_final = self.compile_stmt(*true_blk, context);
                let then_scope = self.scopes.end_scope();
                self.builder.build_unconditional_branch(continue_bb);


                self.builder.position_at_end(else_bb);
                self.scopes.begin_scope();
                let else_bb_final = self.compile_stmt(*false_blk, context);
                self.builder.build_unconditional_branch(continue_bb);
                let else_scope = self.scopes.end_scope();

                self.builder.position_at_end(continue_bb);

                let mut handled = HashSet::new();
                println!("fniishing");
                for (assigned_var, definition) in then_scope.clone().iter().chain(else_scope.clone().iter()) {
                    if handled.contains(assigned_var) { continue }
                    if let Some(existing_defn) = self.scopes.lookup(&assigned_var) {
                        println!("{}", assigned_var);
                        handled.insert(assigned_var.clone());
                        let value_from_if = then_scope.get(assigned_var).or(Some(&existing_defn)).unwrap();
                        let value_from_else = else_scope.get(assigned_var).or(Some(&existing_defn)).unwrap();
                        let phi = self.builder.build_phi(context.f64_type(), &format!("if_else_phi_{}", assigned_var));
                        let value_from_if = value_from_if.as_basic_value_enum().into_float_value();
                        let value_from_else = value_from_else.as_basic_value_enum().into_float_value();
                        phi.add_incoming(&[(&value_from_if, then_bb_final), (&value_from_else, else_bb_final)]);
                        self.scopes.insert(assigned_var.clone(), phi.as_any_value_enum().into_float_value());
                    }
                }
                return continue_bb;
            }
        }
        self.builder.get_insert_block().unwrap()
    }

    fn compile_expr(&mut self, expr: Expr, context: &'ctx Context) -> FloatValue<'ctx> {
        match expr {
            Expr::String(_str) => context.f64_type().const_float(1.0),
            Expr::Number(num) => context.f64_type().const_float(num),
            Expr::BinOp(left, op, right) => {
                let name = self.name();
                let l = self.compile_expr(*left, context);
                let r = self.compile_expr(*right, context);
                match op {
                    BinOp::Minus => self.builder.build_float_sub(l, r, &name),
                    BinOp::Plus => self.builder.build_float_add(l, r, &name),
                    BinOp::Slash => self.builder.build_float_div(l, r, &name),
                    BinOp::Star => self.builder.build_float_mul(l, r, &name),
                    _ => panic!("only arithmetic")
                }
            }
            Expr::Variable(name) => self.scopes.lookup(&name).unwrap(),
            Expr::LogicalOp(_, _, _) => { panic!("logic not done yet") }
        }
    }
}