use std::any::Any;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::execution_engine::{ExecutionEngine};
use inkwell::module::{Linkage, Module};
use inkwell::{AddressSpace, FloatPredicate};
use inkwell::basic_block::BasicBlock;
use inkwell::types::{StructType};
use inkwell::values::{AnyValue, BasicValue, FloatValue, FunctionValue, InstructionOpcode};
use crate::{BinOp, Expr};
use crate::parser::{Stmt, Program, PatternAction};

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

// type BodyFunc = unsafe extern "C" fn(f64) -> f64;

type ScopeInfo<'ctx> = HashMap<String, FloatValue<'ctx>>;

struct Scope<'ctx> {
    pub values: ScopeInfo<'ctx>,
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
    pub fn end_scope(&mut self) -> ScopeInfo<'ctx> {
        self.scopes.pop().unwrap().values
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


        for begin in prog.begins.iter() {
            self.compile_stmt(begin, context);
        }

        for pa in prog.pattern_actions.iter() {
            self.compile_pattern_action(pa, context);
        }

        for end in prog.ends.iter() {
            self.compile_stmt(end, context);
        }


        // If the last instruction isn't a return, add one and return 0.0.
        let zero = context.f64_type().const_float(0.0);
        match self.builder.get_insert_block().unwrap().get_last_instruction() {
            None => { self.builder.build_return(Some(&zero)); } // No instructions in the block
            Some(last) => {
                match last.get_opcode() {
                    InstructionOpcode::Return => {} // it is a return, do nothing
                    _ => { self.builder.build_return(Some(&zero)); }
                }
            }
        };

        let str = self.module.print_to_string().to_string().replace("\\n", "\n");
        self.module.write_bitcode_to_path(Path::new("/tmp/crawk.bc"));
        println!("{}", str);
        // unsafe { self.execution_engine.get_function(ROOT).ok() }.expect("to get root func")
    }

    fn compile_pattern_action(&mut self, pa: &PatternAction, context: &'ctx Context) {
        if let Some(test) = &pa.pattern {

            let action_bb = context.append_basic_block(self.module.get_function(ROOT).expect("root to exist"), "action_bb");
            let continue_bb = context.append_basic_block(self.module.get_function(ROOT).expect("root to exist"), "continue_bb");

            let predicate = self.compile_expr(test, context);
            let zero = context.f64_type().const_float(0.0);
            let comparison = self.builder.build_float_compare(FloatPredicate::OEQ, predicate, zero, "pattern-action-test");

            self.builder.build_conditional_branch(comparison, action_bb, continue_bb);

            self.builder.position_at_end(action_bb);
            self.scopes.begin_scope();
            let action_bb_final = self.compile_stmt(&pa.action, context);
            let action_bb_scope = self.scopes.end_scope();
            self.builder.build_unconditional_branch(continue_bb);

            self.builder.position_at_end(continue_bb);
            self.build_phis(vec![(action_bb, action_bb_scope)], context);
        } else {
            self.compile_stmt(&pa.action, context);
        }
    }

    fn compile_stmt(&mut self, stmt: &Stmt, context: &'ctx Context) -> BasicBlock<'ctx> {
        match stmt {
            Stmt::Expr(expr) => {self.compile_expr(expr, context);},
            Stmt::Print(expr) => {
                let res = self.compile_expr(expr, context);
                self.builder.build_call(self.print_func, &[res.into()], "print_f64_call");
            }
            Stmt::Assign(name, expr) => {
                let fin = self.compile_expr(expr, context);
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
                let then_bb_final = self.compile_stmt(true_blk, context);
                let then_scope = self.scopes.end_scope();
                self.builder.build_unconditional_branch(continue_bb);

                self.builder.position_at_end(else_bb);
                self.scopes.begin_scope();
                let else_bb_final = self.compile_stmt(false_blk, context);
                self.builder.build_unconditional_branch(continue_bb);
                let else_scope = self.scopes.end_scope();

                self.builder.position_at_end(continue_bb);

                self.build_phis(vec![(then_bb_final, then_scope), (else_bb_final, else_scope)], context);

                return continue_bb;
            }
        }
        self.builder.get_insert_block().unwrap()
    }

    fn compile_expr(&mut self, expr: &Expr, context: &'ctx Context) -> FloatValue<'ctx> {
        match expr {
            Expr::String(_str) => context.f64_type().const_float(1.0),
            Expr::Variable(str) => self.scopes.lookup(str).expect("to be defined"), // todo: default value
            Expr::Number(num) => context.f64_type().const_float(*num),
            Expr::BinOp(left, op, right) => {
                let name = self.name();
                let l = self.compile_expr(left, context);
                let r = self.compile_expr(right, context);
                match op {
                    BinOp::Minus => self.builder.build_float_sub(l, r, &name),
                    BinOp::Plus => self.builder.build_float_add(l, r, &name),
                    BinOp::Slash => self.builder.build_float_div(l, r, &name),
                    BinOp::Star => self.builder.build_float_mul(l, r, &name),
                    _ => panic!("only arithmetic")
                }
            }
            Expr::Column(expr) => {
                self.compile_expr(expr, context)
            }
            Expr::LogicalOp(_, _, _) => { panic!("logic not done yet") }
        }
    }

    fn build_phis(&mut self, predecessors: Vec<(BasicBlock<'ctx>, ScopeInfo<'ctx>)>, context: &'ctx Context) {
        let mut handled = HashSet::new();
        let mut variables = vec![];

        for pred in predecessors.iter() {
            for (name, _val) in pred.1.iter() {
                variables.push(name.clone());
            }
        }

        if variables.len() == 0 {
            return;
        }

        for assigned_var in variables {
            if handled.contains(&assigned_var) { continue; }
            handled.insert(assigned_var.clone());
            if let Some(existing_defn) = self.scopes.lookup(&assigned_var) {
                let phi = self.builder.build_phi(context.f64_type(), &format!("phi_for_{}", assigned_var));
                let mut incoming: Vec<(&dyn BasicValue<'ctx>, BasicBlock<'ctx>)> = vec![];
                for (pred_block, pred_scope) in predecessors.iter() {
                    let value_in_block = pred_scope.get(&assigned_var)
                        .or(Some(&existing_defn)).unwrap();
                    incoming.push((value_in_block, *pred_block));
                }
                phi.add_incoming(incoming.as_slice());
                self.scopes.insert(assigned_var, phi.as_any_value_enum().into_float_value())
            }
        }
    }
}