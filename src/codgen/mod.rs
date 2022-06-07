mod scopes;
mod runtime;
mod subroutines;
mod variable_extract;

use std::any::Any;
use std::collections::{HashSet};
use std::path::{Path, PathBuf};
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::execution_engine::{ExecutionEngine, JitFunction};
use inkwell::module::{Linkage, Module};
use inkwell::{AddressSpace, FloatPredicate, IntPredicate, OptimizationLevel};
use inkwell::basic_block::BasicBlock;
use inkwell::memory_buffer::MemoryBuffer;
use inkwell::types::{FloatType, IntType};
use inkwell::values::{AggregateValue, AnyValue, BasicMetadataValueEnum, BasicValue, BasicValueEnum, FloatValue, FunctionValue, InstructionOpcode, IntValue, PointerValue};
use crate::{BinOp, Expr};
use crate::codgen::scopes::{ScopeInfo, Scopes};
use crate::codgen::subroutines::Subroutines;
use crate::codgen::runtime::{pad, Runtime};
use crate::parser::{Stmt};

/// Value type
///
/// tag: u8   (0 is f64, 2 is string)
/// | number f64
/// | string

pub fn compile_and_run(prog: Stmt, files: &[String], dump: bool) {
    let context = Context::create();
    let mut codegen = CodeGen::new(&context);
    codegen.compile(prog, &context, files, dump);
    codegen.run()
}

pub fn compile_to_bc(prog: Stmt, files: &[String], output_path: PathBuf) {
    let context = Context::create();
    let mut codegen = CodeGen::new(&context);
    codegen.compile(prog, &context, files, false);
    codegen.write_to_file(output_path);
}

pub enum Value {
    Float(f64),
    ConstString(String),
}

type RootT = unsafe extern "C" fn() -> i64;

const RUNTIME_BITCODE: &[u8] = std::include_bytes!("../../runtime.bc");

const ROOT: &'static str = "main";
const FLOAT_TAG: u8 = 0;
const STRING_TAG: u8 = 1;
// Should be freed upon overwrite
const CONST_STRING_TAG: u8 = 2; // Should not

struct CodeGen<'ctx> {
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    execution_engine: ExecutionEngine<'ctx>,
    scopes: Scopes<'ctx>,
    runtime: Runtime<'ctx>,
    subroutines: Subroutines<'ctx>,
}

type ValuePtrT<'ctx> = (PointerValue<'ctx>, PointerValue<'ctx>);
type ValueT<'ctx> = (IntValue<'ctx>, FloatValue<'ctx>);

impl<'ctx> CodeGen<'ctx> {
    fn new(context: &'ctx Context) -> Self {
        let module = MemoryBuffer::create_from_file(Path::new("./runtime.bc")).expect("to load runtime");//RUNTIME_BITCODE, "runtime");
        let module = Module::parse_bitcode_from_buffer(&module, context).unwrap();
        // let module = context.create_module("llvm-awk");
        // let module = {
        //     runtime.verify().expect("runtime to veirfy");
        //
        //     module.link_in_module(runtime).expect("failed to link cpp runtime");
        //     module.verify().expect("module with runtime to verify");
        //     module
        // };

        let execution_engine = module.create_execution_engine().expect("To be able to create exec engine");
        // execution_engine.add_module(&module).expect("to be able to add runtime to ee");
        // module.link_in_module(runtime).expect("to be able to link in runtime");
        let runtime = Runtime::new(context, &module);
        let mut builder = context.create_builder();
        let subroutines = Subroutines::new(context, &module, &runtime, &mut builder);

        let codegen = CodeGen {
            module,
            builder,
            execution_engine,
            runtime,
            scopes: Scopes::new(),
            subroutines,
        };
        codegen
    }

    fn run(&mut self) {
        println!("module verifies: {:?}", self.module.verify());
        ExecutionEngine::link_in_mc_jit();
        // self.execution_engine.link_in_mc_jit();
        let main = self.module.get_function(ROOT).unwrap();
        unsafe {
            let res= self.execution_engine.run_function(main, &[]);
            // let func: JitFunction<RootT> = self.execution_engine.get_function(ROOT).ok().expect("ROOT function not found");
            // let res = func.call();
        }
    }
    fn write_to_file(&self, path_buf: PathBuf) {
        if !self.module.write_bitcode_to_path(path_buf.as_path()) {
            panic!("Couldn't write bitcode to path")
        }
    }

    fn compile(&mut self, prog: Stmt, context: &'ctx Context, files: &[String], dump: bool) {
        let i64_type = context.i64_type();
        let i64_func = i64_type.fn_type(&[], false);
        let function = self.module.add_function(ROOT, i64_func, Some(Linkage::External));
        let init_bb = context.append_basic_block(function, "init_bb");
        self.builder.position_at_end(init_bb);

        // Pass list of files over to c++ side at runtime.
        // We could do this with fewer func calls
        self.builder.position_at_end(init_bb);
        for path in files.iter().rev() {
            let mut path = path.to_string();
            pad(&mut path);
            let const_str = context.const_string(path.as_bytes(), true);
            let malloced_array = self.builder.build_alloca(const_str.get_type(), "file_path string alloc");
            self.builder.build_store(malloced_array, const_str);

            let i8_ptr_type = context.i8_type().ptr_type(AddressSpace::Generic);
            let array_as_i8_ptr = self.builder.build_cast(
                InstructionOpcode::BitCast,
                malloced_array,
                i8_ptr_type,
                "cast to i8*");
            self.builder.build_call(self.runtime.add_file, &[array_as_i8_ptr.into()], "add file");
        }
        self.builder.build_call(self.runtime.init, &[], "done adding files call init!");

        self.define_all_vars(&prog, context);
        let final_bb = self.compile_stmt(&prog, context);

        self.builder.position_at_end(final_bb);
        // If the last instruction isn't a return, add one and return 0

        let zero = context.i64_type().const_int(0, false);
        match self.builder.get_insert_block().unwrap().get_last_instruction() {
            None => { self.builder.build_return(Some(&zero)); } // No instructions in the block
            Some(last) => {
                match last.get_opcode() {
                    InstructionOpcode::Return => {} // it is a return, do nothing
                    _ => { self.builder.build_return(Some(&zero)); }
                }
            }
        };

        if dump {
            println!("{}", self.module.print_to_string().to_string().replace("\\n", "\n"));
        }
    }

    fn alloc(&mut self, tag: IntValue<'ctx>, value: FloatValue<'ctx>, context: &'ctx Context) -> (PointerValue<'ctx>, PointerValue<'ctx>) {
        let tag_ptr = self.builder.build_alloca(context.i8_type(), "tag");
        let value_ptr = self.builder.build_alloca(context.f64_type(), "value");
        self.builder.build_store(tag_ptr, tag);
        self.builder.build_store(value_ptr, value);
        (tag_ptr, value_ptr)
    }


    fn create_value(&mut self, value: Value, context: &'ctx Context) -> ValueT<'ctx> {
        let zero_i8 = context.i8_type().const_int(FLOAT_TAG as u64, false);
        let two_i8 = context.i8_type().const_int(CONST_STRING_TAG as u64, false);

        match value {
            Value::Float(num) => {
                let num_f64 = context.f64_type().const_float(num);
                let one_i8 = context.i8_type().const_int(FLOAT_TAG as u64, false);
                (one_i8, num_f64)
            }
            Value::ConstString(value) => {
                let name = format!("const-str-{}", value);
                let global_value = self.builder.build_global_string_ptr(&value, &name);
                let global_ptr = global_value.as_pointer_value();
                let float_ptr = self.cast_ptr_to_float(global_ptr, context);
                (two_i8, float_ptr)
            }
        }
    }

    fn define_all_vars(&mut self, prog: &Stmt, context: &'ctx Context) {
        let vars = variable_extract::extract(prog);
        let zero_i8 = context.i8_type().const_int(0, false);
        let zero = context.f64_type().const_float(0.0);
        for var in vars {
            let ptrs = self.alloc(zero_i8, zero, context);
            self.scopes.insert(var, ptrs);
        }
    }

    fn load(&mut self, value: ValuePtrT<'ctx>) -> (IntValue<'ctx>, FloatValue<'ctx>) {
        let (tag, value) = value;
        let tag = self.builder.build_load(tag, "tag").as_any_value_enum().into_int_value();
        let value = self.builder.build_load(value, "value").as_any_value_enum().into_float_value();
        (tag, value)
    }

    fn compile_to_bool(&mut self, expr: &Expr, context: &'ctx Context) -> IntValue<'ctx> {
        let (tag, value) = self.compile_expr(expr, context);
        let tag_is_zero = self.builder.build_int_compare(IntPredicate::EQ, tag, context.i8_type().const_int(0, false), "tag_is_zero");
        let value_is_zero_f64 = self.builder.build_float_compare(FloatPredicate::OEQ, value, context.f64_type().const_float(0.0), "value_is_zero_f64");
        let zero_f64 = self.builder.build_and(value_is_zero_f64, tag_is_zero, "zero_f64");
        let result = self.builder.build_not(zero_f64, "predicate");
        result
    }

    fn value_for_ffi_ptr(&mut self, result: ValuePtrT<'ctx>) -> Vec<BasicMetadataValueEnum<'ctx>> {
        let (tag, value) = self.load(result);
        return vec![tag.into(), value.into()];
    }
    fn value_for_ffi(&mut self, result: ValueT<'ctx>) -> Vec<BasicMetadataValueEnum<'ctx>> {
        return vec![result.0.into(), result.1.into()];
    }


    fn compile_stmt(&mut self, stmt: &Stmt, context: &'ctx Context) -> BasicBlock<'ctx> {
        match stmt {
            Stmt::While(test, body) => {
                // INIT -> while_test
                // while_test -> while_body, while_continue
                // while_body -> while_test
                // while_continue -> END
                let root = self.module.get_function(ROOT).expect("root to exist");

                let pred = self.builder.get_insert_block().unwrap();
                let while_test_bb = context.append_basic_block(root, "while_test");
                let while_body_bb = context.append_basic_block(root, "while_body");
                let continue_bb = context.append_basic_block(root, "while_continue");

                self.builder.build_unconditional_branch(while_test_bb);
                self.builder.position_at_end(while_test_bb);
                let test_result_bool = self.compile_to_bool(test, context);
                self.builder.build_conditional_branch(test_result_bool, while_body_bb, continue_bb);

                self.builder.position_at_end(while_body_bb);
                self.compile_stmt(body, context);
                self.builder.build_unconditional_branch(while_test_bb);

                self.builder.position_at_end(continue_bb);
                return continue_bb;
            }
            Stmt::Expr(expr) => {
                self.compile_expr(expr, context);
            }
            Stmt::Print(expr) => {
                let result = self.compile_expr(expr, context);
                let result = self.value_for_ffi(result);
                self.builder.build_call(self.runtime.print, &result, "print_value_call");
            }
            Stmt::Assign(name, expr) => {
                let fin = self.compile_expr(expr, context);
                if let Some(existing) = self.scopes.lookup(name) {
                    let args = self.value_for_ffi_ptr(existing);
                    self.builder.build_call(self.subroutines.free_if_string, &args, "call-free-if-str");
                    self.builder.build_store(existing.0, fin.0);
                    self.builder.build_store(existing.1, fin.1);
                } else {
                    panic!("Undefined variable {}", name);
                }
            }
            Stmt::Return(result) => {
                let fin = match result {
                    None => context.i64_type().const_int(0, false),
                    Some(val) => self.compile_to_bool(val, context),
                };
                self.builder.build_return(Some(&fin));
            }
            Stmt::Group(body) => {
                let mut last_bb = None;
                for stmt in body {
                    last_bb = Some(self.compile_stmt(stmt, context));
                }
                if let Some(bb) = last_bb {
                    return bb;
                }
            }
            Stmt::If(test, true_blk, false_blk) => {
                if let Some(false_blk) = false_blk {
                    let then_bb = context.append_basic_block(self.module.get_function(ROOT).expect("root to exist"), "then");
                    let else_bb = context.append_basic_block(self.module.get_function(ROOT).expect("root to exist"), "else");
                    let continue_bb = context.append_basic_block(self.module.get_function(ROOT).expect("root to exist"), "merge");

                    let predicate = self.compile_to_bool(test, context);
                    self.builder.build_conditional_branch(predicate, then_bb, else_bb);

                    self.builder.position_at_end(then_bb);
                    let then_bb_final = self.compile_stmt(true_blk, context);
                    self.builder.build_unconditional_branch(continue_bb);

                    self.builder.position_at_end(else_bb);
                    let else_bb_final = self.compile_stmt(false_blk, context);
                    self.builder.build_unconditional_branch(continue_bb);

                    self.builder.position_at_end(continue_bb);

                    return continue_bb;
                } else {
                    let then_bb = context.append_basic_block(self.module.get_function(ROOT).expect("root to exist"), "then");
                    let continue_bb = context.append_basic_block(self.module.get_function(ROOT).expect("root to exist"), "continue");

                    let predicate = self.compile_to_bool(test, context);
                    self.builder.build_conditional_branch(predicate, then_bb, continue_bb);

                    self.builder.position_at_end(then_bb);
                    let then_bb_final = self.compile_stmt(true_blk, context);
                    self.builder.build_unconditional_branch(continue_bb);

                    self.builder.position_at_end(continue_bb);
                    return continue_bb;
                }
            }
        }
        self.builder.get_insert_block().unwrap()
    }

    fn compile_expr(&mut self, expr: &Expr, context: &'ctx Context) -> ValueT<'ctx> {
        match expr {
            Expr::String(str) => {
                self.create_value(Value::ConstString(str.clone()), context)
            }
            Expr::Variable(str) => {
                self.load(self.scopes.lookup(str).expect("to be defined"))
            } // todo: default value
            Expr::NumberF64(num) => {
                self.create_value(Value::Float(*num), context)
            }
            Expr::BinOp(left, op, right) => {
                let l = self.compile_expr(left, context);
                let r = self.compile_expr(right, context);

                let (l, _l_final_bb) = self.build_to_number(l, context);
                let (r, _r_final_bb) = self.build_to_number(r, context);
                let tag = context.i8_type().const_int(FLOAT_TAG as u64, false);
                let float = self.build_f64_binop(l, r, op, context);
                (tag, float)
            }
            Expr::Column(expr) => {
                let res = self.compile_expr(expr, context);
                let args = self.value_for_ffi(res);
                let float_ptr = self.builder.build_call(self.runtime.column, &args, "get_column");
                let one = context.i8_type().const_int(1, false);
                (one, float_ptr.as_any_value_enum().into_float_value())
            }
            Expr::LogicalOp(_, _, _) => { panic!("logic not done yet") }
            Expr::Call => {
                let next_line_res = self.builder.build_call(self.runtime.next_line, &[], "get_next_line").as_any_value_enum().into_float_value();
                (context.i8_type().const_int(FLOAT_TAG as u64, false), next_line_res)
            }
        }
    }

    #[allow(dead_code)]
    fn cast_float_to_int(&self, float: FloatValue<'ctx>, context: &'ctx Context) -> IntValue<'ctx> {
        self.builder.build_bitcast::<IntType, FloatValue>(
            float, context.i64_type(), "cast-float-to-int").into_int_value()
    }

    fn cast_int_to_float(&self, int: IntValue<'ctx>, context: &'ctx Context) -> FloatValue<'ctx> {
        self.builder.build_bitcast::<FloatType, IntValue>(
            int, context.f64_type().into(), "cast-int-to-float").into_float_value()
    }

    fn cast_ptr_to_float(&self, ptr: PointerValue<'ctx>, context: &'ctx Context) -> FloatValue<'ctx> {
        let int = self.builder.build_ptr_to_int(ptr, context.i64_type(), "ptr-to-int");
        self.cast_int_to_float(int, context)
        // self.builder.build_bitcast::<FloatType, PointerValue>(
        //     ptr, context.f64_type().into(), "cast-int-to-float").into_float_value()
    }

    // #[allow(dead_code)]
    // fn build_phis(&mut self, predecessors: Vec<(BasicBlock<'ctx>, ScopeInfo<'ctx>)>, context: &'ctx Context) {
    //     let mut handled = HashSet::new();
    //     let mut variables = vec![];
    //     for pred in predecessors.iter() {
    //         for (name, _val) in pred.1.iter() {
    //             variables.push(name.clone());
    //         }
    //     }
    //     if variables.len() == 0 {
    //         return;
    //     }
    //     for assigned_var in variables {
    //         if handled.contains(&assigned_var) { continue; }
    //         handled.insert(assigned_var.clone());
    //         if let Some(existing_defn) = self.scopes.lookup(&assigned_var) {
    //             let phi_tag = self.builder.build_phi(context.i8_type(), &format!("phi_{}_tag", assigned_var));
    //             let phi_value = self.builder.build_phi(context.f64_type(), &format!("phi_{}_value", assigned_var));
    //
    //             // let mut incoming_tag_vals = vec![];
    //             let mut incoming = vec![];
    //
    //             let mut incoming_tag: Vec<(&dyn BasicValue<'ctx>, BasicBlock<'ctx>)> = vec![];
    //             let mut incoming_value: Vec<(&dyn BasicValue<'ctx>, BasicBlock<'ctx>)> = vec![];
    //             for (pred_block, pred_scope) in predecessors.iter() {
    //                 let value_in_block = match pred_scope.get(&assigned_var) {
    //                     None => existing_defn,
    //                     Some(val_in_scope) => val_in_scope.clone(),
    //                 };
    //                 incoming.push((value_in_block.0, value_in_block.1, pred_block.clone()));
    //                 // incoming_tag_vals.push((, pred_block.clone()));
    //                 // incoming_tag.push((&value_in_block.0, pred_block.clone()));
    //                 // incoming_value.push((&value_in_block.1, pred_block.clone()));
    //             }
    //             for val in incoming.iter() {
    //                 incoming_tag.push((&val.0, val.2));
    //                 incoming_value.push((&val.1, val.2));
    //             }
    //
    //
    //             phi_tag.add_incoming(incoming_tag.as_slice());
    //             phi_value.add_incoming(incoming_value.as_slice());
    //             self.scopes.insert(assigned_var, (phi_tag.as_any_value_enum().into_int_value(), phi_value.as_any_value_enum().into_float_value()));
    //         }
    //     }
    // }

    fn build_to_number(&mut self, value: ValueT<'ctx>, context: &'ctx Context) -> (FloatValue<'ctx>, BasicBlock<'ctx>) {
        // Get ROOT function
        let root = self.module.get_function(ROOT).expect("root to exist");
        // Basic blocks for not number and number
        let init_bb = self.builder.get_insert_block().unwrap();
        let not_number_bb = context.append_basic_block(root, "not_number");
        let done_bb = context.append_basic_block(root, "done_bb");

        let (tag, value) = value;
        let cmp = self.builder.build_int_compare(IntPredicate::EQ, context.i8_type().const_int(0, false), tag, "is_zero");
        self.builder.build_conditional_branch(cmp, done_bb, not_number_bb);

        self.builder.position_at_end(not_number_bb);
        let args = self.value_for_ffi((tag, value));
        let number: FloatValue = self.builder.build_call(self.runtime.string_to_number, &args, "string_to_number").as_any_value_enum().into_float_value();
        self.builder.build_unconditional_branch(done_bb);

        self.builder.position_at_end(done_bb);
        let phi = self.builder.build_phi(context.f64_type(), "string_to_number_phi");
        let init_value = value.as_basic_value_enum();
        phi.add_incoming(&[(&init_value, init_bb), (&number, not_number_bb)]);
        (phi.as_basic_value().into_float_value(), done_bb)
    }

    fn build_f64_binop(&mut self, left_float: FloatValue<'ctx>, right_float: FloatValue<'ctx>, op: &BinOp, context: &'ctx Context) -> FloatValue<'ctx> {
        let name = "both-f64-binop-tag";
        match op {
            BinOp::Minus => self.builder.build_float_sub(left_float, right_float, name),
            BinOp::Plus => self.builder.build_float_add(left_float, right_float, name),
            BinOp::Slash => self.builder.build_float_div(left_float, right_float, name),
            BinOp::Star => self.builder.build_float_mul(left_float, right_float, name),
            BinOp::Greater | BinOp::Less | BinOp::GreaterEq |
            BinOp::LessEq | BinOp::EqEq | BinOp::BangEq => {
                let predicate = op.predicate();
                let result = self.builder.build_float_compare(predicate, left_float, right_float, name);

                let root = self.module.get_function(ROOT).expect("root to exist");
                let is_zero_bb = context.append_basic_block(root, "binop_zero");
                let is_one_bb = context.append_basic_block(root, "binop_one");
                let continue_bb = context.append_basic_block(root, "binop_cont");

                self.builder.build_conditional_branch(result, is_one_bb, is_zero_bb);

                self.builder.position_at_end(is_one_bb);
                self.builder.build_unconditional_branch(continue_bb);

                self.builder.position_at_end(is_zero_bb);
                self.builder.build_unconditional_branch(continue_bb);

                self.builder.position_at_end(continue_bb);
                let phi = self.builder.build_phi(context.f64_type(), "binop_phi");
                phi.add_incoming(&[(&context.f64_type().const_float(0.0), is_zero_bb), (&context.f64_type().const_float(1.0), is_one_bb)]);
                return phi.as_basic_value().into_float_value();
            }

            _ => panic!("only arithmetic")
        }
    }
}