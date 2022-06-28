mod scopes;
// mod runtime;
// mod subroutines;
mod variable_extract;

use std::os::raw::{c_char, c_void};
use gnu_libjit::{Abi, Context, Function, Label, Value};
use crate::{Expr};
use crate::codgen::scopes::Scopes;
use crate::parser::{Stmt, TypedExpr};
use crate::runtime::{Runtime};

/// Value type
///
/// tag: u8   (0 is f64, 2 is string)
/// | number f64
/// | string

pub fn compile_and_run(prog: Stmt, files: &[String], dump: bool, capture: bool) {
    let mut codegen = CodeGen::new(files.to_vec(), capture);
    codegen.compile(prog, files, dump);
    codegen.run()
}

pub const FLOAT_TAG: u8 = 0;
pub const VAR_STRING_TAG: u8 = 1; // String that comes from a variable
pub const CONST_STRING_TAG: u8 = 2; // String that appears in the program source

// A string that is the result of a temporary computation
// a = "abc" "def"
// abc and def are CONST_STRING_TAG and "abcdef" is TMP_STRING_TAG
pub const TMP_STRING_TAG: u8 = 3;



struct CodeGen {
    function: Function,
    scopes: Scopes,
    context: Context,
    runtime: Runtime,
    binop_scratch: ValuePtrT, // Since we don't have phis just store the result of binops here
}

type ValueT = (Value, Value);
type ValuePtrT = ValueT;

impl CodeGen {
    fn new(files: Vec<String>, capture: bool) -> Self {
        let mut context = Context::new();
        let function = context.function(Abi::Cdecl, Context::float64_type(), vec![]).expect("to create function");
        let runtime = Runtime::new(files, capture);
        let binop_scratch_tag = function.alloca(1);
        let binop_scratch_value = function.alloca(8);
        let binop_scratch = (binop_scratch_tag, binop_scratch_value);
        let codegen = CodeGen {
            function,
            scopes: Scopes::new(),
            context,
            runtime,
            binop_scratch
        };
        codegen
    }

    fn run(&mut self) {
        let function: extern "C" fn(f64) -> f64 = self.function.to_closure();
        // let res = function(123.123);
        todo!("run!")
    }

    fn output(&self) -> String {
        self.runtime.output()
    }

    fn compile(&mut self, prog: Stmt, files: &[String], dump: bool) {
        let zero = self.function.create_float64_constant(0.0);

        self.define_all_vars(&prog);
        self.compile_stmt(&prog);

        self.function.insn_return(&zero);

        self.context.build_end();
        if dump {
            println!("{}", self.function.dump().unwrap());
        }
        self.function.compile();
    }

    fn runtime_data_ptr(&mut self) -> Value {
        self.function.create_void_ptr_constant(self.runtime.data_ptr())
    }

    fn alloc_value(&mut self) -> ValuePtrT {
        let tag = self.function.alloca(1);
        let value = self.function.alloca(8);
        (tag, value)
    }

    fn define_all_vars(&mut self, prog: &Stmt) {
        let vars = variable_extract::extract(prog);

        for var in vars {
            let val = self.alloc_value();
            self.scopes.insert(var, val);
        }
    }

    fn compile_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Expr(expr) => { self.compile_expr(expr); }
            Stmt::Print(expr) => {
                let val = self.compile_expr(expr);
                let ptr = self.runtime_data_ptr();
                self.function.insn_call_native(self.runtime.print_value, vec![ptr, val.0, val.1], None);
            }
            Stmt::Assign(variable, expr) => {
                let val = self.compile_expr(expr);
                let variable_ptr = self.scopes.get(variable);
                self.function.insn_store(&variable_ptr.0, &val.0);
                self.function.insn_store(&variable_ptr.1, &val.1);
            }
            Stmt::Group(group) => {
                for group in group {
                    self.compile_stmt(group)
                }
            }
            Stmt::If(test, if_so, if_not) => {
                let test = self.compile_expr(test);
                let ptr = self.runtime_data_ptr();
                let bool_value = self.function.insn_call_native(self.runtime.is_truthy, vec![ptr, test.0, test.1], Some(Context::int_type()));
                let mut then_label = Label::new();
                let mut done_label = Label::new();
                self.function.insn_branch_if(&bool_value, &mut then_label);
                if let Some(if_not) = if_not {
                    self.compile_stmt(if_not);
                }
                self.function.insn_branch(&mut done_label);
                self.function.insn_label(&mut then_label);
                self.compile_stmt(if_so);
                self.function.insn_branch(&mut done_label);
            }
            Stmt::While(test, body) => {
                let mut test_label = Label::new();
                let mut done_label = Label::new();
                self.function.insn_label(&mut test_label);
                let test = self.compile_expr(test);
                let ptr = self.runtime_data_ptr();
                let bool_value = self.function.insn_call_native(self.runtime.is_truthy, vec![ptr, test.0, test.1], Some(Context::int_type()));
                self.function.insn_branch_if(&bool_value, &mut done_label);
                self.compile_stmt(body);
                self.function.insn_branch(&mut test_label);
                self.function.insn_label(&mut done_label);
            }
        }
    }

    fn compile_expr(&mut self, expr: &TypedExpr) -> ValueT {
        match &expr.expr {
            Expr::NumberF64(num) => (self.function.create_sbyte_constant(FLOAT_TAG as c_char),
                                     self.function.create_float64_constant(*num)),
            Expr::String(str) => {
                let boxed = Box::new(str.to_string());
                let tag = self.function.create_sbyte_constant(CONST_STRING_TAG as c_char);
                let raw_ptr = Box::into_raw(boxed);
                let ptr = self.function.create_void_ptr_constant(raw_ptr as *mut c_void);
                (tag, ptr)
            }
            Expr::MathOp(left, op, right) => {
                todo!("mathop");
                // let left = self.compile_expr(left);
                // let right = self.compile_expr(right);
                // let zero = self.function.create_sbyte_constant(FLOAT_TAG as c_char);
                // let one = self.function.create_sbyte_constant(VAR_STRING_TAG as c_char);
                // let two = self.function.create_sbyte_constant(CONST_STRING_TAG as c_char);
                //
                // let l_is_float = self.function.insn_eq(&left.0, &zero);
                // let r_is_float = self.function.insn_eq(&right.0, &zero);
                // let l_is_reg_string = self.function.insn_eq(&left.0, &one);
                // let r_is_reg_string = self.function.insn_eq(&right.0, &one);
                // let l_is_const_string = self.function.insn_eq(&left.0, &two);
                // let r_is_const_string = self.function.insn_eq(&right.0, &two);
                // let l_is_str = self.function.insn_or(&l_is_reg_string, &l_is_const_string);
                // let r_is_str = self.function.insn_or(&r_is_reg_string, &r_is_const_string);
                //
                // // Result of math op is put here
                // let result = self.alloc_value();
                // let done_label = Label::new();
                //
                // // DONE
                // let tag = self.function.insn_load(&result.0);
                // let value = self.function.insn_load(&result.1);
                //
                //
                //
                // let mut done_lbl = Label::new();
            }
            Expr::BinOp(left, op, right) => {
                let left = self.compile_expr(left);
                let right = self.compile_expr(right);
                todo!("binop")
            }
            Expr::LogicalOp(left, op, right) => {
                todo!("logical op")
            }
            Expr::Variable(var) => {
                let var_ptr = self.scopes.get(var);
                let tag = self.function.insn_load(&var_ptr.0);
                let val = self.function.insn_load(&var_ptr.1);
                (tag, val)
            }
            Expr::Column(col) => {
                let column = self.compile_expr(col);
                let ptr = self.runtime_data_ptr();
                let val = self.function.insn_call_native(self.runtime.column, vec![ptr, column.0, column.1], Some(Context::float64_type()));
                let tag = self.function.create_sbyte_constant(VAR_STRING_TAG as c_char);
                (tag, val)
            }
            Expr::Call => {
                todo!("what is call for ?")
            }
        }
    }

    //
    // fn load(&mut self, value: ValuePtrT<'ctx>) -> (IntValue<'ctx>, FloatValue<'ctx>) {
    //     let (tag, value) = value;
    //     let tag = self.builder.build_load(tag, "tag").as_any_value_enum().into_int_value();
    //     let value = self.builder.build_load(value, "value").as_any_value_enum().into_float_value();
    //     (tag, value)
    // }
    //
    // fn compile_to_bool(&mut self, expr: &Expr, context: &'ctx Context) -> IntValue<'ctx> {
    //     let (tag, value) = self.compile_expr(expr, context);
    //     let tag_is_zero = self.builder.build_int_compare(IntPredicate::EQ, tag, context.i8_type().const_int(0, false), "tag_is_zero");
    //     let value_is_zero_f64 = self.builder.build_float_compare(FloatPredicate::OEQ, value, context.f64_type().const_float(0.0), "value_is_zero_f64");
    //     let zero_f64 = self.builder.build_and(value_is_zero_f64, tag_is_zero, "zero_f64");
    //     let result = self.builder.build_not(zero_f64, "predicate");
    //     result
    // }
    //
    // fn value_for_ffi_ptr(&mut self, result: ValuePtrT<'ctx>) -> Vec<BasicMetadataValueEnum<'ctx>> {
    //     let (tag, value) = self.load(result);
    //     return vec![tag.into(), value.into()];
    // }
    // fn value_for_ffi(&mut self, result: ValueT<'ctx>) -> Vec<BasicMetadataValueEnum<'ctx>> {
    //     return vec![result.0.into(), result.1.into()];
    // }
    //
    //
    // fn compile_stmt(&mut self, stmt: &Stmt, context: &'ctx Context) -> BasicBlock<'ctx> {
    //     match stmt {
    //         Stmt::While(test, body) => {
    //             // INIT -> while_test
    //             // while_test -> while_body, while_continue
    //             // while_body -> while_test
    //             // while_continue -> END
    //             let root = self.module.get_function(ROOT).expect("root to exist");
    //
    //             let pred = self.builder.get_insert_block().unwrap();
    //             let while_test_bb = context.append_basic_block(root, "while_test");
    //             let while_body_bb = context.append_basic_block(root, "while_body");
    //             let continue_bb = context.append_basic_block(root, "while_continue");
    //
    //             self.builder.build_unconditional_branch(while_test_bb);
    //             self.builder.position_at_end(while_test_bb);
    //             let test_result_bool = self.compile_to_bool(test, context);
    //             self.builder.build_conditional_branch(test_result_bool, while_body_bb, continue_bb);
    //
    //             self.builder.position_at_end(while_body_bb);
    //             self.compile_stmt(body, context);
    //             self.builder.build_unconditional_branch(while_test_bb);
    //
    //             self.builder.position_at_end(continue_bb);
    //             return continue_bb;
    //         }
    //         Stmt::Expr(expr) => {
    //             self.compile_expr(expr, context);
    //         }
    //         Stmt::Print(expr) => {
    //             let result = self.compile_expr(expr, context);
    //             let result = self.value_for_ffi(result);
    //             self.builder.build_call(self.runtime.print, &result, "print_value_call");
    //         }
    //         Stmt::Assign(name, expr) => {
    //             let fin = self.compile_expr(expr, context);
    //             if let Some(existing) = self.scopes.lookup(name) {
    //                 let args = self.value_for_ffi_ptr(existing);
    //                 self.builder.build_call(self.subroutines.free_if_string, &args, "call-free-if-str");
    //                 self.builder.build_store(existing.0, fin.0);
    //                 self.builder.build_store(existing.1, fin.1);
    //             } else {
    //                 panic!("Undefined variable {}", name);
    //             }
    //         }
    //         Stmt::Return(result) => {
    //             let fin = match result {
    //                 None => context.i64_type().const_int(0, false),
    //                 Some(val) => self.compile_to_bool(val, context),
    //             };
    //             self.builder.build_return(Some(&fin));
    //         }
    //         Stmt::Group(body) => {
    //             let mut last_bb = None;
    //             for stmt in body {
    //                 last_bb = Some(self.compile_stmt(stmt, context));
    //             }
    //             if let Some(bb) = last_bb {
    //                 return bb;
    //             }
    //         }
    //         Stmt::If(test, true_blk, false_blk) => {
    //             if let Some(false_blk) = false_blk {
    //                 let then_bb = context.append_basic_block(self.module.get_function(ROOT).expect("root to exist"), "then");
    //                 let else_bb = context.append_basic_block(self.module.get_function(ROOT).expect("root to exist"), "else");
    //                 let continue_bb = context.append_basic_block(self.module.get_function(ROOT).expect("root to exist"), "merge");
    //
    //                 let predicate = self.compile_to_bool(test, context);
    //                 self.builder.build_conditional_branch(predicate, then_bb, else_bb);
    //
    //                 self.builder.position_at_end(then_bb);
    //                 let then_bb_final = self.compile_stmt(true_blk, context);
    //                 self.builder.build_unconditional_branch(continue_bb);
    //
    //                 self.builder.position_at_end(else_bb);
    //                 let else_bb_final = self.compile_stmt(false_blk, context);
    //                 self.builder.build_unconditional_branch(continue_bb);
    //
    //                 self.builder.position_at_end(continue_bb);
    //
    //                 return continue_bb;
    //             } else {
    //                 let then_bb = context.append_basic_block(self.module.get_function(ROOT).expect("root to exist"), "then");
    //                 let continue_bb = context.append_basic_block(self.module.get_function(ROOT).expect("root to exist"), "continue");
    //
    //                 let predicate = self.compile_to_bool(test, context);
    //                 self.builder.build_conditional_branch(predicate, then_bb, continue_bb);
    //
    //                 self.builder.position_at_end(then_bb);
    //                 let then_bb_final = self.compile_stmt(true_blk, context);
    //                 self.builder.build_unconditional_branch(continue_bb);
    //
    //                 self.builder.position_at_end(continue_bb);
    //                 return continue_bb;
    //             }
    //         }
    //     }
    //     self.builder.get_insert_block().unwrap()
    // }
    //
    // fn compile_expr(&mut self, expr: &Expr, context: &'ctx Context) -> ValueT<'ctx> {
    //     match expr {
    //         Expr::String(str) => {
    //             self.create_value(Value::ConstString(str.clone()), context)
    //         }
    //         Expr::Variable(str) => {
    //             self.load(self.scopes.lookup(str).expect("to be defined"))
    //         } // todo: default value
    //         Expr::NumberF64(num) => {
    //             self.create_value(Value::Float(*num), context)
    //         }
    //         Expr::BinOp(left, op, right) => {
    //             let l = self.compile_expr(left, context);
    //             let r = self.compile_expr(right, context);
    //
    //             let (l, _l_final_bb) = self.build_to_number(l, context);
    //             let (r, _r_final_bb) = self.build_to_number(r, context);
    //             let tag = context.i8_type().const_int(FLOAT_TAG as u64, false);
    //             let float = self.build_f64_binop(l, r, op, context);
    //             (tag, float)
    //         }
    //         Expr::Column(expr) => {
    //             let res = self.compile_expr(expr, context);
    //             let args = self.value_for_ffi(res);
    //             let float_ptr = self.builder.build_call(self.runtime.column, &args, "get_column");
    //             let one = context.i8_type().const_int(1, false);
    //             (one, float_ptr.as_any_value_enum().into_float_value())
    //         }
    //         Expr::LogicalOp(_, _, _) => { panic!("logic not done yet") }
    //         Expr::Call => {
    //             let next_line_res = self.builder.build_call(self.runtime.next_line, &[], "get_next_line").as_any_value_enum().into_float_value();
    //             (context.i8_type().const_int(FLOAT_TAG as u64, false), next_line_res)
    //         }
    //     }
    // }
    //
    // #[allow(dead_code)]
    // fn cast_float_to_int(&self, float: FloatValue<'ctx>, context: &'ctx Context) -> IntValue<'ctx> {
    //     self.builder.build_bitcast::<IntType, FloatValue>(
    //         float, context.i64_type(), "cast-float-to-int").into_int_value()
    // }
    //
    // fn cast_int_to_float(&self, int: IntValue<'ctx>, context: &'ctx Context) -> FloatValue<'ctx> {
    //     self.builder.build_bitcast::<FloatType, IntValue>(
    //         int, context.f64_type().into(), "cast-int-to-float").into_float_value()
    // }
    //
    // fn cast_ptr_to_float(&self, ptr: PointerValue<'ctx>, context: &'ctx Context) -> FloatValue<'ctx> {
    //     let int = self.builder.build_ptr_to_int(ptr, context.i64_type(), "ptr-to-int");
    //     self.cast_int_to_float(int, context)
    //     // self.builder.build_bitcast::<FloatType, PointerValue>(
    //     //     ptr, context.f64_type().into(), "cast-int-to-float").into_float_value()
    // }
    //
    // // #[allow(dead_code)]
    // // fn build_phis(&mut self, predecessors: Vec<(BasicBlock<'ctx>, ScopeInfo<'ctx>)>, context: &'ctx Context) {
    // //     let mut handled = HashSet::new();
    // //     let mut variables = vec![];
    // //     for pred in predecessors.iter() {
    // //         for (name, _val) in pred.1.iter() {
    // //             variables.push(name.clone());
    // //         }
    // //     }
    // //     if variables.len() == 0 {
    // //         return;
    // //     }
    // //     for assigned_var in variables {
    // //         if handled.contains(&assigned_var) { continue; }
    // //         handled.insert(assigned_var.clone());
    // //         if let Some(existing_defn) = self.scopes.lookup(&assigned_var) {
    // //             let phi_tag = self.builder.build_phi(context.i8_type(), &format!("phi_{}_tag", assigned_var));
    // //             let phi_value = self.builder.build_phi(context.f64_type(), &format!("phi_{}_value", assigned_var));
    // //
    // //             // let mut incoming_tag_vals = vec![];
    // //             let mut incoming = vec![];
    // //
    // //             let mut incoming_tag: Vec<(&dyn BasicValue<'ctx>, BasicBlock<'ctx>)> = vec![];
    // //             let mut incoming_value: Vec<(&dyn BasicValue<'ctx>, BasicBlock<'ctx>)> = vec![];
    // //             for (pred_block, pred_scope) in predecessors.iter() {
    // //                 let value_in_block = match pred_scope.get(&assigned_var) {
    // //                     None => existing_defn,
    // //                     Some(val_in_scope) => val_in_scope.clone(),
    // //                 };
    // //                 incoming.push((value_in_block.0, value_in_block.1, pred_block.clone()));
    // //                 // incoming_tag_vals.push((, pred_block.clone()));
    // //                 // incoming_tag.push((&value_in_block.0, pred_block.clone()));
    // //                 // incoming_value.push((&value_in_block.1, pred_block.clone()));
    // //             }
    // //             for val in incoming.iter() {
    // //                 incoming_tag.push((&val.0, val.2));
    // //                 incoming_value.push((&val.1, val.2));
    // //             }
    // //
    // //
    // //             phi_tag.add_incoming(incoming_tag.as_slice());
    // //             phi_value.add_incoming(incoming_value.as_slice());
    // //             self.scopes.insert(assigned_var, (phi_tag.as_any_value_enum().into_int_value(), phi_value.as_any_value_enum().into_float_value()));
    // //         }
    // //     }
    // // }
    //
    // fn build_to_number(&mut self, value: ValueT<'ctx>, context: &'ctx Context) -> (FloatValue<'ctx>, BasicBlock<'ctx>) {
    //     // Get ROOT function
    //     let root = self.module.get_function(ROOT).expect("root to exist");
    //     // Basic blocks for not number and number
    //     let init_bb = self.builder.get_insert_block().unwrap();
    //     let not_number_bb = context.append_basic_block(root, "not_number");
    //     let done_bb = context.append_basic_block(root, "done_bb");
    //
    //     let (tag, value) = value;
    //     let cmp = self.builder.build_int_compare(IntPredicate::EQ, context.i8_type().const_int(0, false), tag, "is_zero");
    //     self.builder.build_conditional_branch(cmp, done_bb, not_number_bb);
    //
    //     self.builder.position_at_end(not_number_bb);
    //     let args = self.value_for_ffi((tag, value));
    //     let number: FloatValue = self.builder.build_call(self.runtime.string_to_number, &args, "string_to_number").as_any_value_enum().into_float_value();
    //     self.builder.build_unconditional_branch(done_bb);
    //
    //     self.builder.position_at_end(done_bb);
    //     let phi = self.builder.build_phi(context.f64_type(), "string_to_number_phi");
    //     let init_value = value.as_basic_value_enum();
    //     phi.add_incoming(&[(&init_value, init_bb), (&number, not_number_bb)]);
    //     (phi.as_basic_value().into_float_value(), done_bb)
    // }
    //
    // fn build_f64_binop(&mut self, left_float: FloatValue<'ctx>, right_float: FloatValue<'ctx>, op: &BinOp, context: &'ctx Context) -> FloatValue<'ctx> {
    //     let name = "both-f64-binop-tag";
    //     match op {
    //         BinOp::Minus => self.builder.build_float_sub(left_float, right_float, name),
    //         BinOp::Plus => self.builder.build_float_add(left_float, right_float, name),
    //         BinOp::Slash => self.builder.build_float_div(left_float, right_float, name),
    //         BinOp::Star => self.builder.build_float_mul(left_float, right_float, name),
    //         BinOp::Greater | BinOp::Less | BinOp::GreaterEq |
    //         BinOp::LessEq | BinOp::EqEq | BinOp::BangEq => {
    //             let predicate = op.predicate();
    //             let result = self.builder.build_float_compare(predicate, left_float, right_float, name);
    //
    //             let root = self.module.get_function(ROOT).expect("root to exist");
    //             let is_zero_bb = context.append_basic_block(root, "binop_zero");
    //             let is_one_bb = context.append_basic_block(root, "binop_one");
    //             let continue_bb = context.append_basic_block(root, "binop_cont");
    //
    //             self.builder.build_conditional_branch(result, is_one_bb, is_zero_bb);
    //
    //             self.builder.position_at_end(is_one_bb);
    //             self.builder.build_unconditional_branch(continue_bb);
    //
    //             self.builder.position_at_end(is_zero_bb);
    //             self.builder.build_unconditional_branch(continue_bb);
    //
    //             self.builder.position_at_end(continue_bb);
    //             let phi = self.builder.build_phi(context.f64_type(), "binop_phi");
    //             phi.add_incoming(&[(&context.f64_type().const_float(0.0), is_zero_bb), (&context.f64_type().const_float(1.0), is_one_bb)]);
    //             return phi.as_basic_value().into_float_value();
    //         }
    //
    //         _ => panic!("only arithmetic")
    //     }
}