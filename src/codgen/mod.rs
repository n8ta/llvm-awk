mod scopes;
mod types;
mod subroutines;
mod variable_extract;

use std::collections::{HashSet};
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::execution_engine::{ExecutionEngine};
use inkwell::module::{Linkage, Module};
use inkwell::{FloatPredicate, IntPredicate, OptimizationLevel};
use inkwell::basic_block::BasicBlock;
use inkwell::memory_buffer::MemoryBuffer;
use inkwell::types::{FloatType, IntType};
use inkwell::values::{AggregateValue, AnyValue, BasicMetadataValueEnum, BasicValue, BasicValueEnum, FloatValue, FunctionValue, InstructionOpcode, IntValue, PointerValue, StructValue};
use crate::{BinOp, Expr};
use crate::codgen::scopes::{ScopeInfo, Scopes};
use crate::codgen::subroutines::Subroutines;
use crate::codgen::types::{pad, Types};
use crate::parser::{Stmt};

/// Value type
///
/// tag: u8   (0 is i64, 1 is f64)
/// | number i64
/// | number f64

pub fn compile(prog: Stmt, files: &[String], dump: bool) -> MemoryBuffer {
    let context = Context::create();
    let mut codegen = CodeGen::new(&context);
    codegen.compile(prog, &context, files, dump)
}

pub enum Value {
    Int(i64),
    Float(f64),
}

const ROOT: &'static str = "main";

struct CodeGen<'ctx> {
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    execution_engine: ExecutionEngine<'ctx>,
    counter: usize,
    scopes: Scopes<'ctx>,
    types: Types<'ctx>,
    subroutines: Subroutines<'ctx>,
}

// type RootFunc = unsafe extern "C" fn() -> i32;

impl<'ctx> CodeGen<'ctx> {
    fn new(context: &'ctx Context) -> Self {
        let module = context.create_module("llvm-awk");
        let execution_engine = module.create_jit_execution_engine(OptimizationLevel::Default).expect("To be able to create exec engine");
        let types = Types::new(context, &module);
        let mut builder = context.create_builder();
        let subroutines = Subroutines::new(context, &module, &types, &mut builder);
        let codegen = CodeGen {
            module,
            builder,
            execution_engine,
            counter: 0,
            types,
            scopes: Scopes::new(),
            subroutines,
        };
        codegen
    }

    fn create_value(&mut self, value: Value, context: &'ctx Context) -> StructValue<'ctx> {
        let result = self.builder.build_alloca(self.types.value, "new_non_const_value");
        let tag_ptr = self.builder.build_struct_gep(result, 0, "get-tag-field").unwrap();
        let value_ptr = self.builder.build_struct_gep(result, 1, "get-value-field").unwrap();

        let zero_i8 = context.i8_type().const_int(0, false);
        let one_i8 = context.i8_type().const_int(1, false);

        match value {
            Value::Int(num) => {
                self.builder.build_store(tag_ptr, zero_i8);
                self.builder.build_store(value_ptr, context.i64_type().const_int(num as u64, false));
            },
            Value::Float(num) => {
                let val = unsafe {
                    std::mem::transmute::<f64, u64>(num)
                };
                self.builder.build_store(tag_ptr, one_i8);
                self.builder.build_store(value_ptr, context.i64_type().const_int(val, false));
            }
        }
        self.builder.build_load(result, "load_new_value").into_struct_value()
    }

    fn name(&mut self) -> String {
        self.counter += 1;
        format!("tmp{}", self.counter)
    }
    fn compile(&mut self, prog: Stmt, context: &'ctx Context, files: &[String], dump: bool) -> MemoryBuffer {
        let i64_type = context.i64_type();
        let i64_func = i64_type.fn_type(&[], false);
        let function = self.module.add_function(ROOT, i64_func, Some(Linkage::External));
        let init_bb = context.append_basic_block(function, "init_bb");

        // Pass list of files over to c++ side at runtime.
        // We could do this with fewer func calls
        self.builder.position_at_end(init_bb);
        for path in files.iter().rev() {
            let mut path = path.to_string();
            pad(&mut path);
            let const_str = context.const_string(path.as_bytes(), true);
            let malloced_array = self.builder.build_alloca(const_str.get_type(), "file_path string alloc");
            self.builder.build_store(malloced_array, const_str);
            self.builder.build_call(self.types.add_file, &[malloced_array.into()], "add file");
        }
        self.builder.build_call(self.types.init, &[], "done adding files call init!");

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
        // unsafe {
        //     println!("getting root func");
        //     let root: JitFunction<RootFunc> = self.execution_engine.get_function(ROOT).unwrap();
        //     println!("calling root func");
        //     root.call();
        // }
        self.module.write_bitcode_to_memory()
    }

    fn compile_to_bool(&mut self, expr: &Expr, context: &'ctx Context) -> IntValue<'ctx> {
        let result = self.compile_expr(expr, context);
        let tag = self.builder.build_extract_value(result, 0, "extract tag").unwrap().into_int_value();
        let value = self.builder.build_extract_value(result, 1, "extract value").unwrap().into_int_value();

        let tag_is_zero = self.builder.build_int_compare(IntPredicate::EQ, tag, context.i8_type().const_int(0, false), "tag_is_zero");
        let tag_is_one = self.builder.build_int_compare(IntPredicate::EQ, tag, context.i8_type().const_int(1, false), "tag_is_one");
        let tag_is_two = self.builder.build_int_compare(IntPredicate::EQ, tag, context.i8_type().const_int(2, false), "tag_is_two");
        let value_is_one_i64 = self.builder.build_int_compare(IntPredicate::EQ, value, context.i64_type().const_int(1, false), "value_is_one_i64");
        let value_as_float = self.cast_int_to_float(value, context);
        let value_is_one_f64 = self.builder.build_float_compare(FloatPredicate::OEQ, value_as_float, context.f64_type().const_float(1.0), "value_is_one_f64");
        let one_i64 = self.builder.build_and(tag_is_zero, value_is_one_i64, "is_one_i64");
        let one_f64 = self.builder.build_and(tag_is_one, value_is_one_f64, "is_one_f64");
        let is_one = self.builder.build_or(one_f64, one_i64, "is_one");
        let is_truthy = self.builder.build_or(is_one, tag_is_two, "is_one_or_str");
        is_truthy
    }

    // fn compile_pattern_action(&mut self, pa: &PatternAction, context: &'ctx Context) -> BasicBlock<'ctx> {
    //     if let Some(test) = &pa.pattern {
    //         let action_bb = context.append_basic_block(self.module.get_function(ROOT).expect("root to exist"), "action_bb");
    //         let continue_bb = context.append_basic_block(self.module.get_function(ROOT).expect("root to exist"), "continue_bb");
    //
    //
    //         let predicate = self.compile_to_bool(test, context);
    //         let zero = context.i64_type().const_int(0, false);
    //         let comparison = self.builder.build_int_compare(IntPredicate::EQ, predicate, zero, "pattern-action-test");
    //
    //         self.builder.build_conditional_branch(comparison, action_bb, continue_bb);
    //
    //         self.builder.position_at_end(action_bb);
    //         self.scopes.begin_scope();
    //         let action_bb_final = self.compile_stmt(&pa.action, context);
    //         let action_bb_scope = self.scopes.end_scope();
    //         self.builder.build_unconditional_branch(continue_bb);
    //
    //         self.builder.position_at_end(continue_bb);
    //         self.build_phis(vec![(action_bb_final, action_bb_scope)], context);
    //         continue_bb
    //     } else {
    //         self.compile_stmt(&pa.action, context)
    //     }
    // }

    fn value_for_ffi(&self, result: StructValue<'ctx>) -> Vec<BasicMetadataValueEnum<'ctx>> {
        let f0 = self.builder.build_extract_value(result, 0, "field1").unwrap();
        let f1 = self.builder.build_extract_value(result, 1, "field2").unwrap();
        return vec![f0.into(), f1.into()];
    }

    fn compile_stmt(&mut self, stmt: &Stmt, context: &'ctx Context) -> BasicBlock<'ctx> {
        match stmt {
            Stmt::While(test, body) => {
                // pred -> test_block
                // test_block -> passes, continue
                // while_body -> while_body_final
                // while_body_final -> test_block
                let root = self.module.get_function(ROOT).expect("root to exist");

                let pred = self.builder.get_insert_block().unwrap();
                let test_block_bb = context.append_basic_block(root, "while_test_block");
                let while_body_bb = context.append_basic_block(root, "while_body");
                let continue_bb = context.append_basic_block(root, "while_continue");

                self.builder.build_unconditional_branch(test_block_bb);
                self.builder.position_at_end(test_block_bb);
                let test_result_bool = self.compile_to_bool(test, context);
                self.builder.build_conditional_branch(test_result_bool, while_body_bb, continue_bb);

                self.scopes.begin_scope();
                self.builder.position_at_end(while_body_bb);
                let while_body_final_bb = self.compile_stmt(body, context);
                let while_body_scope = self.scopes.end_scope();
                self.builder.build_unconditional_branch(test_block_bb);


                self.builder.position_at_end(test_block_bb);
                self.build_phis(vec![(pred, ScopeInfo::new()), (while_body_final_bb, while_body_scope)], context);
                return continue_bb;
            }
            Stmt::Expr(expr) => { self.compile_expr(expr, context); }
            Stmt::Print(expr) => {
                let result = self.compile_expr(expr, context);
                let args = self.value_for_ffi(result);
                self.builder.build_call(self.types.print, &args, "print_value_call");
            }
            Stmt::Assign(name, expr) => {
                let fin = self.compile_expr(expr, context);
                let val = self.scopes.lookup(name).unwrap();
                self.builder.build_call(self.subroutines.free_if_string, &[val.into()], "assigning to existing val, free if needed");
                self.scopes.insert(name.clone(), fin);
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
                let false_blk = if let Some(fal) = false_blk { fal } else { panic!("must have false block") };

                let then_bb = context.append_basic_block(self.module.get_function(ROOT).expect("root to exist"), "then");
                let else_bb = context.append_basic_block(self.module.get_function(ROOT).expect("root to exist"), "else");
                let continue_bb = context.append_basic_block(self.module.get_function(ROOT).expect("root to exist"), "merge");

                let predicate = self.compile_to_bool(test, context);
                self.builder.build_conditional_branch(predicate, then_bb, else_bb);

                self.builder.position_at_end(then_bb);
                self.scopes.begin_scope();
                let then_bb_final = self.compile_stmt(true_blk, context);
                let then_scope = self.scopes.end_scope();
                self.builder.build_unconditional_branch(continue_bb);

                self.builder.position_at_end(else_bb);
                self.scopes.begin_scope();
                let else_bb_final = self.compile_stmt(false_blk, context);
                let else_scope = self.scopes.end_scope();
                self.builder.build_unconditional_branch(continue_bb);

                self.builder.position_at_end(continue_bb);

                self.build_phis(vec![(then_bb_final, then_scope), (else_bb_final, else_scope)], context);

                return continue_bb;
            }
        }
        self.builder.get_insert_block().unwrap()
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

    fn new_non_const_value(&self, tag: IntValue<'ctx>, value: IntValue<'ctx>) -> StructValue<'ctx> {
        let result = self.builder.build_alloca(self.types.value, "new_non_const_value");
        let tag_ptr = self.builder.build_struct_gep(result, 0, "get-tag-field").unwrap();
        let value_ptr = self.builder.build_struct_gep(result, 1, "get-value-field").unwrap();
        self.builder.build_store(tag_ptr, tag);
        self.builder.build_store(value_ptr, value);
        self.builder.build_load(result, "load_new_value").into_struct_value()
    }

    fn compile_expr(&mut self, expr: &Expr, context: &'ctx Context) -> StructValue<'ctx> {
        match expr {
            Expr::String(_str) => todo!("Strings"),
            Expr::Variable(str) => self.scopes.lookup(str).expect("to be defined"), // todo: default value
            Expr::NumberF64(num) => {
                self.create_value(Value::Float(*num), context)
            }
            Expr::NumberI64(num) => {
                self.create_value(Value::Int(*num), context)
            }
            Expr::BinOp(left, op, right) => {
                let zero_i64 = context.i64_type().const_int(0, true);
                let zero_i8 = context.i8_type().const_int(0, false); // sign extension doesn't matter it's positive
                let one_i8 = context.i8_type().const_int(1, false);

                let l = self.compile_expr(left, context);
                let r = self.compile_expr(right, context);

                let left_tag = self.builder.build_extract_value(l, 0, "left_tag").unwrap().into_int_value();
                let right_tag = self.builder.build_extract_value(r, 0, "left_tag").unwrap().into_int_value();

                let left_is_f64 = self.builder.build_int_compare(IntPredicate::EQ, left_tag, one_i8, "left_is_f64");
                let right_is_f64 = self.builder.build_int_compare(IntPredicate::EQ, right_tag, one_i8, "left_is_f64");
                let left_is_i64 = self.builder.build_int_compare(IntPredicate::EQ, left_tag, zero_i8, "left_is_i64");
                let right_is_i64 = self.builder.build_int_compare(IntPredicate::EQ, right_tag, zero_i8, "right_is_i64");

                let both_f64 = self.builder.build_and(left_is_f64, right_is_f64, "both_f64");
                let both_i64 = self.builder.build_and(left_is_i64, right_is_i64, "both_i64");

                // Blocks:
                // start -> both_f64 or not_both_f64
                // both_f64 --> continue
                // not_both_f64 --> both_i64 or mismatch
                // both_i64 --> continue
                // mismatch
                // continue

                let both_f64_bb = context.append_basic_block(self.module.get_function(ROOT).expect("root to exist"), "both_f64_binop");
                let not_both_f64_bb = context.append_basic_block(self.module.get_function(ROOT).expect("root to exist"), "not_both_f64_binop");
                let both_i64_bb = context.append_basic_block(self.module.get_function(ROOT).expect("root to exist"), "both_i64_binop");
                let mismatch_bb = context.append_basic_block(self.module.get_function(ROOT).expect("root to exist"), "mismatch_binop");
                let continue_bb = context.append_basic_block(self.module.get_function(ROOT).expect("root to exist"), "continue_binop");

                self.builder.build_conditional_branch(both_f64, both_f64_bb, not_both_f64_bb);

                let both_f64_result = {
                    // Both f64 basic block --> continue or not_both_f64
                    self.builder.position_at_end(both_f64_bb);
                    let left_float = self.builder.build_extract_value(l, 1, "left_float").unwrap().into_int_value();
                    let right_float = self.builder.build_extract_value(r, 1, "right_float").unwrap().into_int_value();

                    let left_float = self.cast_int_to_float(left_float, context);
                    let right_float = self.cast_int_to_float(right_float, context);

                    let name = format!("{}{}", self.name(), "-both-f64-binop-tag");
                    let res = match op {
                        BinOp::Minus => self.builder.build_float_sub(left_float, right_float, &name),
                        BinOp::Plus => self.builder.build_float_add(left_float, right_float, &name),
                        BinOp::Slash => self.builder.build_float_div(left_float, right_float, &name),
                        BinOp::Star => self.builder.build_float_mul(left_float, right_float, &name),
                        _ => panic!("only arithmetic")
                    };

                    let res_as_i64: IntValue = self.builder.build_bitcast::<IntType, FloatValue>(
                        res, context.i64_type().into(), "cast-float-back-to-i64").into_int_value();

                    let result = self.new_non_const_value(context.i8_type().const_int(1, false), res_as_i64);
                    self.builder.build_unconditional_branch(continue_bb);
                    result
                };

                self.builder.position_at_end(not_both_f64_bb);
                self.builder.build_conditional_branch(both_i64, both_i64_bb, mismatch_bb);

                // Not both f64 basic block  ---> both_i64 or mismatch
                self.builder.position_at_end(both_i64_bb);

                let both_i64_result = {
                    let left_int = self.builder.build_extract_value(l, 1, "left_float").unwrap().into_int_value();
                    let right_int = self.builder.build_extract_value(r, 1, "right_float").unwrap().into_int_value();
                    let name = format!("{}{}", self.name(), "-both-i64-binop");
                    let res = match op {
                        BinOp::Minus => self.builder.build_int_sub(left_int, right_int, &name),
                        BinOp::Plus => self.builder.build_int_add(left_int, right_int, &name),
                        BinOp::Slash => self.builder.build_int_signed_div(left_int, right_int, &name),
                        BinOp::Star => self.builder.build_int_mul(left_int, right_int, &name),
                        _ => panic!("only arithmetic")
                    };

                    let res = self.new_non_const_value(zero_i8, res);
                    self.builder.build_unconditional_branch(continue_bb);
                    res
                };

                // Return -1 if types mismatch
                self.builder.position_at_end(mismatch_bb);
                self.builder.build_call(self.types.mismatch, &[], "call mismatch print");
                self.builder.build_return(Some(&zero_i64));

                self.builder.position_at_end(continue_bb);
                let phi = self.builder.build_phi(self.types.value, "tagged_enum_expr_result_phi");


                let mut incoming: Vec<(&dyn BasicValue<'ctx>, BasicBlock<'ctx>)> = vec![];
                incoming.push((&both_i64_result, both_i64_bb));
                incoming.push((&both_f64_result, both_f64_bb));
                phi.add_incoming(incoming.as_slice());
                phi.as_basic_value().into_struct_value()
            }
            Expr::Column(expr) => {
                let res = self.compile_expr(expr, context);
                let tag = self.builder.build_extract_value(res, 0, "tag for col").unwrap().into_int_value();
                let val = self.builder.build_extract_value(res, 1, "value for col").unwrap().into_int_value();
                let int_ptr = self.builder.build_call(self.types.column, &[tag.into(), val.into()], "get_column").as_any_value_enum().into_int_value();
                self.new_non_const_value(context.i8_type().const_int(2, false), int_ptr)
            }
            Expr::LogicalOp(_, _, _) => { panic!("logic not done yet") }
            Expr::Call => {
                let next_line_res = self.builder.build_call(self.types.next_line, &[], "get_next_line").as_any_value_enum().into_int_value();
                self.new_non_const_value(context.i8_type().const_int(0, false), next_line_res)
            }
        }
    }

    fn build_phis(&mut self, predecessors: Vec<(BasicBlock<'ctx>, ScopeInfo<'ctx>)>, _context: &'ctx Context) {
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
                let phi = self.builder.build_phi(self.types.value.clone(), &format!("phi_for_{}", assigned_var));
                let mut incoming: Vec<(&dyn BasicValue<'ctx>, BasicBlock<'ctx>)> = vec![];
                for (pred_block, pred_scope) in predecessors.iter() {
                    let value_in_block = match pred_scope.get(&assigned_var) {
                        None => &existing_defn,
                        Some(val_in_scope) => val_in_scope,
                    };
                    incoming.push((value_in_block, *pred_block));
                }
                phi.add_incoming(incoming.as_slice());
                self.scopes.insert(assigned_var, phi.as_any_value_enum().into_struct_value());
            }
        }
    }
}