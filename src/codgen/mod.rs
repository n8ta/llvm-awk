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
use inkwell::values::{AggregateValue, AnyValue, BasicMetadataValueEnum, BasicValue, BasicValueEnum, FloatValue, FunctionValue, InstructionOpcode, IntValue, PointerValue};
use crate::{BinOp, Expr};
use crate::codgen::scopes::{ScopeInfo, Scopes};
use crate::codgen::subroutines::Subroutines;
use crate::codgen::types::{pad, Types};
use crate::parser::{Stmt};

/// Value type
///
/// tag: u8   (0 is f64, 2 is string)
/// | number f64
/// | string

pub fn compile(prog: Stmt, files: &[String], dump: bool) -> MemoryBuffer {
    let context = Context::create();
    let mut codegen = CodeGen::new(&context);
    codegen.compile(prog, &context, files, dump)
}

pub enum Value {
    Float(f64),
}

const ROOT: &'static str = "main";
const FLOAT_TAG: u8 = 0;
const STRING_TAG: u8 = 1;

struct CodeGen<'ctx> {
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    execution_engine: ExecutionEngine<'ctx>,
    scopes: Scopes<'ctx>,
    types: Types<'ctx>,
    subroutines: Subroutines<'ctx>,
}

type ValueT<'ctx> = (IntValue<'ctx>, FloatValue<'ctx>);

impl<'ctx> CodeGen<'ctx> {
    fn new(context: &'ctx Context) -> Self {
        let module = context.create_module("llvm-awk");
        let execution_engine = module.create_jit_execution_engine(OptimizationLevel::Aggressive).expect("To be able to create exec engine");
        let types = Types::new(context, &module);
        let mut builder = context.create_builder();
        let subroutines = Subroutines::new(context, &module, &types, &mut builder);
        let codegen = CodeGen {
            module,
            builder,
            execution_engine,
            types,
            scopes: Scopes::new(),
            subroutines,
        };
        codegen
    }

    fn create_value(&mut self, value: Value, context: &'ctx Context) -> ValueT<'ctx> {
        let zero_i8 = context.i8_type().const_int(0, false);
        match value {
            Value::Float(num) => {
                (zero_i8, context.f64_type().const_float(num))
            }
        }
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

        println!("{}", self.module.print_to_string().to_string());

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
        let tag = result.0;
        let value= result.1;
        let tag_is_zero = self.builder.build_int_compare(IntPredicate::EQ, tag, context.i8_type().const_int(0, false), "tag_is_zero");
        let value_is_zero_f64 = self.builder.build_float_compare(FloatPredicate::OEQ, value, context.f64_type().const_float(0.0), "value_is_zero_f64");
        let zero_f64 = self.builder.build_and(value_is_zero_f64, tag_is_zero, "zero_f64");
        // TODO: string truthyness
        self.builder.build_not(zero_f64, "not_zero_is_truthy")
    }

    fn value_for_ffi(&self, result: ValueT<'ctx>) -> Vec<BasicMetadataValueEnum<'ctx>> {
        return vec![result.0.into(), result.1.into()];
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
                let while_test_bb = context.append_basic_block(root, "while_test_block");
                let while_body_bb = context.append_basic_block(root, "while_body");
                let continue_bb = context.append_basic_block(root, "while_continue");

                self.builder.build_unconditional_branch(while_test_bb);
                self.builder.position_at_end(while_test_bb);
                let test_result_bool = self.compile_to_bool(test, context);
                self.builder.build_conditional_branch(test_result_bool, while_body_bb, continue_bb);

                self.scopes.begin_scope();
                self.builder.position_at_end(while_body_bb);
                let while_body_final_bb = self.compile_stmt(body, context);
                let while_body_scope = self.scopes.end_scope();
                self.builder.build_unconditional_branch(while_test_bb);

                self.builder.position_at_end(continue_bb);
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
                if let Some(existing) = self.scopes.lookup(name) {
                    self.builder.build_call(self.subroutines.free_if_string, &[existing.0.into(), existing.1.into()], "call-free-if-str");
                };
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

    fn build_to_number(&mut self, value: ValueT<'ctx>, context: &'ctx Context) -> (FloatValue<'ctx>, BasicBlock<'ctx>) {
        // Get ROOT function
        let root = self.module.get_function(ROOT).expect("root to exist");
        // Basic blocks for not number and number
        let init_bb = self.builder.get_insert_block().unwrap();
        let not_number_bb = context.append_basic_block(root, "not_number");
        let done_bb = context.append_basic_block(root, "done_bb");

        let cmp = self.builder.build_int_compare(IntPredicate::EQ, context.i8_type().const_int(0, false), value.0, "is_zero");
        self.builder.build_conditional_branch(cmp, done_bb, not_number_bb);

        self.builder.position_at_end(not_number_bb);
        let number: FloatValue = self.builder.build_call(self.types.string_to_number, &[value.0.into(), value.1.into()], "string_to_number").as_any_value_enum().into_float_value();
        self.builder.build_unconditional_branch(done_bb);

        self.builder.position_at_end(done_bb);
        let phi = self.builder.build_phi(context.f64_type(), "string_to_number_phi");
        let init_value = value.1.as_basic_value_enum();
        let number_value = number.as_basic_value_enum();
        phi.add_incoming(&[(&init_value, init_bb), (&number, not_number_bb)]);
        (phi.as_basic_value().into_float_value(), done_bb)
    }

    fn compile_expr(&mut self, expr: &Expr, context: &'ctx Context) -> ValueT<'ctx> {
        match expr {
            Expr::String(_str) => todo!("Strings"),
            Expr::Variable(str) => self.scopes.lookup(str).expect("to be defined"), // todo: default value
            Expr::NumberF64(num) => {
                self.create_value(Value::Float(*num), context)
            }
            Expr::BinOp(left, op, right) => {
                let l = self.compile_expr(left, context);
                let r = self.compile_expr(right, context);

                let (l, l_final_bb) = self.build_to_number(l, context);
                let (r, r_final_bb) = self.build_to_number(r, context);
                self.build_f64_binop(l, r, op, context)
            }
            Expr::Column(expr) => {
                let res = self.compile_expr(expr, context);
                let float_ptr = self.builder.build_call(self.types.column, &self.value_for_ffi(res), "get_column");
                (context.i8_type().const_int(1, false), float_ptr.as_any_value_enum().into_float_value())
            }
            Expr::LogicalOp(_, _, _) => { panic!("logic not done yet") }
            Expr::Call => {
                let next_line_res = self.builder.build_call(self.types.next_line, &[], "get_next_line").as_any_value_enum().into_int_value();
                (context.i8_type().const_int(0, false), self.cast_int_to_float(next_line_res, context))
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
                let phi_tag = self.builder.build_phi(context.i8_type(), &format!("phi_for_{}_tag", assigned_var));
                let phi_value = self.builder.build_phi(context.f64_type(), &format!("phi_for_{}_value", assigned_var));

                // let mut incoming_tag_vals = vec![];
                let mut incoming = vec![];

                let mut incoming_tag: Vec<(&dyn BasicValue<'ctx>, BasicBlock<'ctx>)> = vec![];
                let mut incoming_value: Vec<(&dyn BasicValue<'ctx>, BasicBlock<'ctx>)> = vec![];
                for (pred_block, pred_scope) in predecessors.iter() {
                    let value_in_block = match pred_scope.get(&assigned_var) {
                        None => existing_defn,
                        Some(val_in_scope) => val_in_scope.clone(),
                    };
                    incoming.push((value_in_block.0, value_in_block.1, pred_block.clone()));
                    // incoming_tag_vals.push((, pred_block.clone()));
                    // incoming_tag.push((&value_in_block.0, pred_block.clone()));
                    // incoming_value.push((&value_in_block.1, pred_block.clone()));
                }
                for val in incoming.iter() {
                    incoming_tag.push((&val.0, val.2));
                    incoming_value.push((&val.1, val.2));
                }


                phi_tag.add_incoming(incoming_tag.as_slice());
                phi_value.add_incoming(incoming_value.as_slice());
                self.scopes.insert(assigned_var, (phi_tag.as_any_value_enum().into_int_value(), phi_value.as_any_value_enum().into_float_value()));
            }
        }
    }

    fn build_f64_binop(&mut self, left_float: FloatValue<'ctx>, right_float: FloatValue<'ctx>, op: &BinOp, context: &'ctx Context) -> ValueT<'ctx> {

        let name = "both-f64-binop-tag";
        let res = match op {
            BinOp::Minus => self.builder.build_float_sub(left_float, right_float, name),
            BinOp::Plus => self.builder.build_float_add(left_float, right_float, name),
            BinOp::Slash => self.builder.build_float_div(left_float, right_float, name),
            BinOp::Star => self.builder.build_float_mul(left_float, right_float, name),
            _ => panic!("only arithmetic")
        };

        (context.i8_type().const_int(FLOAT_TAG as u64, false), res)
    }
}