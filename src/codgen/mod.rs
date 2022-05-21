use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::execution_engine::{ExecutionEngine, JitFunction};
use inkwell::module::Module;
use inkwell::OptimizationLevel;
use inkwell::values::FloatValue;
use crate::{BinOp, Expr};

pub fn compile_and_run(expr: Expr) {
    let context = Context::create();
    let mut codegen = CodeGen::new(&context);

    let root: JitFunction<BodyFunc> = codegen.compile(expr, &context);

    unsafe {
        println!("=> {}", root.call());
    }

}
const ROOT: &'static str = "root";

type BodyFunc = unsafe extern "C" fn() -> f64;

struct CodeGen<'ctx> {
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    execution_engine: ExecutionEngine<'ctx>,
    counter: usize,
}

impl<'ctx> CodeGen<'ctx> {
    fn new(context: &'ctx Context) -> Self {
        let module = context.create_module("sum");
        let execution_engine = module.create_jit_execution_engine(OptimizationLevel::None).expect("To be able to create exec engine");
        let codegen = CodeGen {
            module,
            builder: context.create_builder(),
            execution_engine,
            counter: 0,
        };
        codegen
    }
    fn name(&mut self) -> String {
        self.counter += 1;
        format!("tmp{}", self.counter)
    }

    fn compile(&mut self, expr: Expr, context: &'ctx Context) -> JitFunction<BodyFunc> {
        let f64_type = context.f64_type();
        let f64_func = f64_type.fn_type(&[], false);
        let function = self.module.add_function(ROOT, f64_func, None);
        let bb = context.append_basic_block(function, ROOT);
        self.builder.position_at_end(bb);

        let fin = self.compile_expr(expr, context);


        self.builder.build_return(Some(&fin));

        unsafe { self.execution_engine.get_function(ROOT).ok() }.expect("to get root func")
    }

    fn compile_expr(&mut self, expr: Expr, context: &'ctx Context) -> FloatValue<'ctx> {
        match expr {
            Expr::Number(num) => {
                context.f64_type().const_float(num)
            }
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
            Expr::LogicalOp(_, _, _) => { panic!("logic not done yet") }
        }
    }
}