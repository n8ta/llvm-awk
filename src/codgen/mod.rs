use std::any::Any;
use std::collections::HashMap;
use std::path::Path;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::execution_engine::{ExecutionEngine, JitFunction};
use inkwell::module::{Linkage, Module};
use inkwell::{FloatPredicate, OptimizationLevel};
use inkwell::types::{StructType};
use inkwell::values::{FloatValue, PointerValue, CallSiteValue, CallableValue, FunctionValue};
use crate::{BinOp, Expr};
use crate::parser::{Program, Stmt};


// Make this callable from C

#[no_mangle]
pub extern "C" fn print_float_64(num: f64) {
    println!("{}", num);
}

pub fn compile_and_run(prog: Program) {
    let context = Context::create();
    let mut codegen = CodeGen::new(&context);
    codegen.compile(prog, &context);
    // let root: JitFunction<BodyFunc> = ;
    // let ptr = print_float_64 as *const fn(f64);
    // let ptr_to_f64 = unsafe {
    //     std::mem::transmute::<*const fn(f64), *const f64>(ptr)
    // };
    // unsafe {
    //     println!("=> {}", root.call(*ptr_to_f64))
    // }
}

const ROOT: &'static str = "main";

type BodyFunc = unsafe extern "C" fn(f64) -> f64;

struct CodeGen<'ctx> {
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    execution_engine: ExecutionEngine<'ctx>,
    counter: usize,
    value_type: StructType<'ctx>,
    scope: Vec<HashMap<String, FloatValue<'ctx>>>,
    print_func: FunctionValue<'ctx>
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
            scope: vec![HashMap::new()],
            print_func,
        };
        codegen
    }
    fn name(&mut self) -> String {
        self.counter += 1;
        format!("tmp{}", self.counter)
    }

    fn lookup(&self, name: &str) -> FloatValue<'ctx> {
        for scope in self.scope.iter().rev() {
            if let Some(value) = scope.get(name) {
                return value.clone();
            }
        }
        panic!("Unable to find value for {}", name);
    }
    fn begin_scope(&mut self) {
        self.scope.push(HashMap::new())
    }
    fn end_scope(&mut self) {
        self.scope.pop();
    }

    fn compile(&mut self, prog: Program, context: &'ctx Context)  {
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

    fn compile_stmt(&mut self, stmt: Stmt, context: &'ctx Context) {
        match stmt {
            Stmt::Expr(_) => panic!("cannot compile expression stmt"),
            Stmt::Print(expr) => {
                let res = self.compile_expr(expr, context);
                self.builder.build_call(self.print_func, &[res.into()], "print_f64_call");
            }
            Stmt::Assign(name, expr) => {
                let fin = self.compile_expr(expr, context);
                // todo: check if name is already in scope SSA!
                self.scope.last_mut().unwrap().insert(name.clone(), fin);
            }
            Stmt::Return(result) => {
                let fin = match result {
                    None => context.f64_type().const_float(0.0),
                    Some(val) => self.compile_expr(val, context),
                };
                self.builder.build_return(Some(&fin));
            }
            Stmt::Group(body) => {
                self.begin_scope();
                for stmt in body {
                    self.compile_stmt(stmt, context);
                }
                self.end_scope();
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
                self.compile_stmt(*true_blk, context);
                self.builder.build_unconditional_branch(continue_bb);

                self.builder.position_at_end(else_bb);
                self.compile_stmt(*false_blk, context);
                self.builder.build_unconditional_branch(continue_bb);
                self.builder.position_at_end(continue_bb);
            }
        }
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
            Expr::Variable(name) => self.lookup(&name),
            Expr::LogicalOp(_, _, _) => { panic!("logic not done yet") }
        }
    }
}