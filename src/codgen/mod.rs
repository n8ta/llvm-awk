mod scopes;
// mod runtime;
// mod subroutines;
mod variable_extract;

use std::mem::size_of;
use std::os::raw::{c_char, c_double, c_long, c_void};
use gnu_libjit::{Abi, Context, Function, Label, Value};
use crate::{Expr};
use crate::codgen::scopes::Scopes;
use crate::lexer::{BinOp, LogicalOp, MathOp};
use crate::parser::{AwkT, Stmt, TypedExpr};
use crate::runtime::{LiveRuntime, Runtime, TestRuntime};

/// Value type
///
/// tag: u8   (0 is f64, 2 is string)
/// | number f64
/// | string *mut String

pub fn compile_and_run(prog: Stmt, files: &[String]) {
    let mut runtime = LiveRuntime::new(files.to_vec());
    let mut codegen = CodeGen::new(&mut runtime);
    codegen.compile(prog, false);
    codegen.run();
}

pub fn compile_and_capture(prog: Stmt, files: &[String]) -> String {
    let mut test_runtime = TestRuntime::new(files.to_vec());
    let mut codegen = CodeGen::new(&mut test_runtime);
    codegen.compile(prog, true);
    codegen.run();
    test_runtime.output()
}

pub const FLOAT_TAG: u8 = 0;
pub const STRING_TAG: u8 = 1;

struct CodeGen<'a, RuntimeT: Runtime> {
    function: Function,
    scopes: Scopes,
    context: Context,
    runtime: &'a mut RuntimeT,
    binop_scratch: ValuePtrT,
    // Since we don't have phis just store the result of binops here
    binop_scratch_int: Value,
    zero_ptr: Value, // Used to init the pointer section of the value struct
}

#[derive(Clone)]
pub struct ValueT {
    pub tag: Value,
    pub float: Value,
    pub pointer: Value,
}

impl ValueT {
    pub fn new(tag: Value, float: Value, pointer: Value) -> ValueT { ValueT { tag, float, pointer } }
}

type ValuePtrT = ValueT;

impl<'a, RuntimeT: Runtime> CodeGen<'a, RuntimeT> {
    fn new(runtime: &'a mut RuntimeT) -> Self {
        let mut context = Context::new();
        let mut function = context.function(Abi::Cdecl, Context::float64_type(), vec![]).expect("to create function");
        let binop_scratch_tag = function.create_value_int();

        let binop_scratch_float = function.create_value_float64();
        let binop_scratch_int = function.create_value_int();
        let binop_scratch_ptr = function.create_value_void_ptr();

        let zero = Box::new(0);
        let zero_ptr = (Box::leak(zero) as *mut i32) as *mut c_void;
        let zero_ptr = function.create_void_ptr_constant(zero_ptr);

        let binop_scratch = ValueT::new(binop_scratch_tag, binop_scratch_float, binop_scratch_ptr);
        let codegen = CodeGen {
            function,
            scopes: Scopes::new(),
            context,
            runtime,
            binop_scratch,
            binop_scratch_int,
            zero_ptr,
        };
        codegen
    }

    fn run(&mut self) {
        let function: extern "C" fn() = self.function.to_closure();
        function();
    }

    fn compile(&mut self, prog: Stmt, dump: bool) {
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

    fn create_value(&mut self) -> ValuePtrT {
        let tag = self.function.create_value_int();
        let value = self.function.create_value_float64();
        let ptr = self.function.create_value_void_ptr();

        let zero = self.function.create_float64_constant(0 as c_double);
        let float_tag = self.function.create_sbyte_constant(FLOAT_TAG as c_char);

        self.function.insn_store(&tag, &float_tag);
        self.function.insn_store(&value, &zero);
        self.function.insn_store(&ptr, &self.zero_ptr.clone());
        ValueT::new(tag, value, ptr)
    }

    fn define_all_vars(&mut self, prog: &Stmt) {
        for var in variable_extract::extract(prog) {
            let val = self.create_value();
            self.scopes.insert(var, val);
        }
    }

    fn float_is_truthy_ret_int(&mut self, value: &Value) -> Value {
        let zero_f = self.function.create_float64_constant(0.0);
        self.function.insn_ne(&value, &zero_f)
    }

    fn string_is_truthy_ret_int(&mut self, value: &Value) -> Value {
        // Rust string memory layout: [pointer: pointer, cap: usize, len: usize]
        // Grab the length to determine truthyness

        let string_len_offset = size_of::<usize>() + size_of::<*const u8>();
        let string_len = self.function.insn_load_relative(&value, string_len_offset as c_long, &Context::ulong_type());
        let zero_ulong = self.function.create_ulong_constant(0);
        self.function.insn_ne(&zero_ulong, &string_len)
    }

    fn truthy_ret_integer(&mut self, value: &ValueT, typ: AwkT) -> Value {
        match typ {
            AwkT::String => {
                self.string_is_truthy_ret_int(&value.pointer)
            }
            AwkT::Float => {
                self.float_is_truthy_ret_int(&value.float)
            }
            AwkT::Variable => {
                let mut string_lbl = Label::new();
                let mut done_lbl = Label::new();

                let one_tag = self.function.create_sbyte_constant(STRING_TAG as c_char);
                let tag_is_one = self.function.insn_eq(&value.tag, &one_tag);
                self.function.insn_branch_if(&tag_is_one, &mut string_lbl);
                // is float code
                let is_truthy_f = self.float_is_truthy_ret_int(&value.float);
                self.function.insn_store(&self.binop_scratch_int, &is_truthy_f);
                self.function.insn_branch(&mut done_lbl);

                self.function.insn_label(&mut string_lbl);
                let is_truthy_str = self.string_is_truthy_ret_int(&value.pointer);
                self.function.insn_store(&self.binop_scratch_int, &is_truthy_str);
                self.function.insn_label(&mut done_lbl);
                self.function.insn_load(&self.binop_scratch_int)
            }
        }
    }

    fn to_float(&mut self, value: ValueT) -> Value {
        let zero = self.function.create_sbyte_constant(FLOAT_TAG as c_char);

        let mut done_lbl = Label::new();
        self.function.insn_store(&self.binop_scratch.float, &value.float);
        let is_float = self.function.insn_eq(&value.tag, &zero);
        self.function.insn_branch_if(&is_float, &mut done_lbl);

        let ptr = self.runtime_data_ptr();
        let res = self.function.insn_call_native(self.runtime.string_to_number(), vec![ptr, value.pointer], Some(Context::float64_type()));
        self.function.insn_store(&self.binop_scratch.float, &res);

        self.function.insn_label(&mut done_lbl);
        self.function.insn_load(&self.binop_scratch.float)
    }

    fn drop_if_string_ptr(&mut self, value: &ValuePtrT, typ: AwkT) {
        if let AwkT::Float = typ {
            return;
        }
        let value = self.load(&value);
        self.drop_if_str(&value, typ)
    }

    fn drop_if_str(&mut self, value: &ValueT, typ: AwkT) {
        let ptr = self.runtime_data_ptr();
        match typ {
            AwkT::String => {
                self.function.insn_call_native(self.runtime.free_string(), vec![ptr, value.pointer.clone()], Some(Context::float64_type()));
            }
            AwkT::Variable => {
                let str_tag = self.function.create_sbyte_constant(STRING_TAG as c_char);
                let mut done_lbl = Label::new();
                let is_string = self.function.insn_ne(&str_tag, &value.tag);
                self.function.insn_branch_if(&is_string, &mut done_lbl);
                self.function.insn_call_native(self.runtime.free_string(), vec![ptr, value.pointer.clone()], Some(Context::float64_type()));
                self.function.insn_label(&mut done_lbl);
            }
            _ => {}
        };
    }

    fn compile_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Expr(expr) => {
                let res = self.compile_expr(expr);
                self.drop_if_str(&res, expr.typ);
            }
            Stmt::Print(expr) => {
                let val = self.compile_expr(expr);
                let ptr = self.runtime_data_ptr();
                match expr.typ {
                    AwkT::String => {
                        self.function.insn_call_native(self.runtime.print_string(), vec![ptr, val.pointer.clone()], None);
                    }
                    AwkT::Float => {
                        self.function.insn_call_native(self.runtime.print_float(), vec![ptr, val.float.clone()], None);
                    }
                    AwkT::Variable => {
                        let zero_tag = self.function.create_sbyte_constant(FLOAT_TAG as c_char);
                        let mut float_lbl = Label::new();
                        let mut done_lbl = Label::new();
                        let tag_is_zero = self.function.insn_eq(&zero_tag, &val.tag);
                        self.function.insn_branch_if(&tag_is_zero, &mut float_lbl);
                        self.function.insn_call_native(self.runtime.print_string(), vec![ptr.clone(), val.pointer.clone()], None);
                        self.function.insn_branch(&mut done_lbl);
                        self.function.insn_label(&mut float_lbl);
                        self.function.insn_call_native(self.runtime.print_float(), vec![ptr, val.float.clone()], None);
                        self.function.insn_label(&mut done_lbl);
                    }
                }
                // self.drop_if_str(&val, expr.typ);
            }
            Stmt::Group(group) => {
                for group in group {
                    self.compile_stmt(group)
                }
            }
            Stmt::If(test, if_so, if_not) => {
                let test_value = self.compile_expr(test);
                let bool_value = self.truthy_ret_integer(&test_value, test.typ);
                // self.drop_if_str(&test_value, test.typ);
                let mut then_label = Label::new();
                let mut done_label = Label::new();

                // branch_if(test) :then_label
                // if_not_section
                // go_to :done_label
                // :then_label
                // if_so_section
                // :done_label

                self.function.insn_branch_if(&bool_value, &mut then_label);
                if let Some(if_not) = if_not {
                    self.compile_stmt(if_not);
                    self.function.insn_branch(&mut done_label);
                }
                self.function.insn_label(&mut then_label);
                self.compile_stmt(if_so);
                self.function.insn_label(&mut done_label);
            }
            Stmt::While(test, body) => {
                let mut test_label = Label::new();
                let mut done_label = Label::new();
                self.function.insn_label(&mut test_label);
                let test_value = self.compile_expr(test);
                let bool_value = self.truthy_ret_integer(&test_value, test.typ);
                // self.drop_if_str(&test_value, test.typ);
                self.function.insn_branch_if_not(&bool_value, &mut done_label);
                self.compile_stmt(body);
                self.function.insn_branch(&mut test_label);
                self.function.insn_label(&mut done_label);
            }
        }
    }

    fn copy_if_string(&mut self, value: ValueT, typ: AwkT) -> ValueT {
        let zero = self.function.create_float64_constant(0.0);
        let str_tag = self.function.create_sbyte_constant(STRING_TAG as c_char);
        let data_ptr = self.runtime_data_ptr();
        match typ {
            AwkT::String => {
                let ptr = self.function.insn_call_native(self.runtime.copy_string(), vec![data_ptr, value.pointer], Some(Context::void_ptr_type()));
                ValueT::new(str_tag, zero, ptr)
            }
            AwkT::Float => value,
            AwkT::Variable => {
                let mut done = Label::new();
                let is_string = self.function.insn_eq(&str_tag, &value.tag);
                self.function.insn_store(&self.binop_scratch.pointer, &self.zero_ptr);
                self.function.insn_branch_if_not(&is_string, &mut done);
                let ptr = self.function.insn_call_native(self.runtime.copy_string(), vec![data_ptr, value.pointer], Some(Context::void_ptr_type()));
                self.function.insn_store(&self.binop_scratch.pointer, &ptr);
                self.function.insn_label(&mut done);
                let string = self.function.insn_load(&self.binop_scratch.pointer);
                ValueT::new(value.tag, value.float, string)
            }
        }
    }

    fn compile_expr(&mut self, expr: &TypedExpr) -> ValueT {
        match &expr.expr {
            Expr::Assign(var, value) => {
                let new_value = self.compile_expr(value);
                let var_ptrs = self.scopes.get(var).clone();

                let old_value = self.load(&var_ptrs);
                self.drop_if_str(&old_value, AwkT::Variable);

                self.function.insn_store(&var_ptrs.tag, &new_value.tag);
                self.function.insn_store(&var_ptrs.float, &new_value.float);
                self.function.insn_store(&var_ptrs.pointer, &new_value.pointer);

                self.copy_if_string(new_value, value.typ)
            }
            Expr::NumberF64(num) =>
                ValueT::new(
                    self.function.create_sbyte_constant(FLOAT_TAG as c_char),
                    self.function.create_float64_constant(*num),
                    self.zero_ptr.clone()),
            Expr::String(str) => {
                let boxed = Box::new(str.to_string());
                let tag = self.function.create_sbyte_constant(STRING_TAG as c_char);
                let raw_ptr = unsafe { Box::into_raw(boxed) };
                let ptr = self.function.create_void_ptr_constant(raw_ptr as *mut c_void);
                ValueT::new(tag, self.function.create_float64_constant(0.0), ptr)
            }
            Expr::MathOp(left_expr, op, right_expr) => {
                let mut left = self.compile_expr(left_expr);
                let mut right = self.compile_expr(right_expr);
                let zero = self.function.create_sbyte_constant(FLOAT_TAG as c_char);

                if AwkT::Float != left_expr.typ {
                    left = ValueT::new(zero.clone(), self.to_float(left), self.zero_ptr.clone());
                }
                if AwkT::Float != right_expr.typ {
                    right = ValueT::new(zero.clone(), self.to_float(right), self.zero_ptr.clone());
                }
                let res = match op {
                    MathOp::Minus => self.function.insn_sub(&left.float, &right.float),
                    MathOp::Plus => self.function.insn_add(&left.float, &right.float),
                    MathOp::Slash => self.function.insn_div(&left.float, &right.float),
                    MathOp::Star => self.function.insn_mult(&left.float, &right.float),
                };

                // self.drop_if_str(&left, left_expr.typ);
                // self.drop_if_str(&right, left_expr.typ);

                ValueT::new(zero, res, self.zero_ptr.clone())
            }
            Expr::BinOp(left_expr, op, right_expr) => {
                let left = self.compile_expr(left_expr);
                let right = self.compile_expr(right_expr);
                let tag = self.function.create_sbyte_constant(FLOAT_TAG as c_char);
                let value = match (left_expr.typ, right_expr.typ) {
                    (AwkT::Float, AwkT::Float) => {
                        match op {
                            BinOp::Greater => self.function.insn_gt(&left.float, &right.float),
                            BinOp::GreaterEq => self.function.insn_ge(&left.float, &right.float),
                            BinOp::Less => self.function.insn_lt(&left.float, &right.float),
                            BinOp::LessEq => self.function.insn_le(&left.float, &right.float),
                            BinOp::BangEq => self.function.insn_ne(&left.float, &right.float),
                            BinOp::EqEq => self.function.insn_eq(&left.float, &right.float),
                            BinOp::MatchedBy => todo!("matched expr"),
                            BinOp::NotMatchedBy => todo!("matched expr"),
                        }
                    }
                    _ => {
                        todo!("non float float binop ")
                    }
                };
                // value is currently an integer, convert to float
                let one = self.function.create_int_constant(0);
                let one_f = self.function.create_float64_constant(1.0);
                let zero_f = self.function.create_float64_constant(0.0);

                let is_one = self.function.insn_eq(&one, &value);
                let mut is_one_lbl = Label::new();
                let mut done_lbl = Label::new();
                self.function.insn_branch_if(&is_one, &mut is_one_lbl);
                self.function.insn_store(&self.binop_scratch.float, &one_f);
                self.function.insn_branch(&mut done_lbl);
                self.function.insn_label(&mut is_one_lbl);
                self.function.insn_store(&self.binop_scratch.float, &zero_f);
                self.function.insn_label(&mut done_lbl);

                // self.drop_if_str(&left, left_expr.typ);
                // self.drop_if_str(&right, right_expr.typ);

                ValueT::new(tag, self.function.insn_load(&self.binop_scratch.float), self.zero_ptr.clone())
            }
            Expr::LogicalOp(left, op, right) => {
                let float_1 = self.function.create_float64_constant(1.0);
                let float_0 = self.function.create_float64_constant(0.0);
                let res = match op {
                    LogicalOp::And => {
                        let mut ret_false = Label::new();
                        let mut done = Label::new();
                        let left_val = self.compile_expr(left);
                        let l = self.truthy_ret_integer(&left_val, left.typ);
                        self.function.insn_branch_if_not(&l, &mut ret_false);
                        let right_val = self.compile_expr(right);
                        let r = self.truthy_ret_integer(&right_val, right.typ);
                        self.function.insn_branch_if_not(&r, &mut ret_false);
                        self.function.insn_store(&self.binop_scratch.float, &float_1);
                        self.function.insn_branch(&mut done);
                        self.function.insn_label(&mut ret_false);
                        self.function.insn_store(&self.binop_scratch.float, &float_0);
                        self.function.insn_branch(&mut done);
                        self.function.insn_label(&mut done);
                        let tag = self.function.create_sbyte_constant(FLOAT_TAG as c_char);
                        let result_f = self.function.insn_load(&self.binop_scratch.float);
                        ValueT::new(tag, result_f, self.zero_ptr.clone())
                    }
                    LogicalOp::Or => {
                        let mut done = Label::new();
                        let mut return_true = Label::new();
                        let l = self.compile_expr(left);
                        let l = self.truthy_ret_integer(&l, left.typ);
                        self.function.insn_branch_if(&l, &mut return_true);
                        let r = self.compile_expr(right);
                        let r = self.truthy_ret_integer(&r, left.typ);
                        self.function.insn_branch_if(&r, &mut return_true);
                        self.function.insn_store(&self.binop_scratch.float, &float_0);
                        self.function.insn_branch(&mut done);
                        self.function.insn_label(&mut return_true);
                        self.function.insn_store(&self.binop_scratch.float, &float_1);
                        self.function.insn_label(&mut done);
                        let tag = self.function.create_sbyte_constant(FLOAT_TAG as c_char);
                        let result_f = self.function.insn_load(&self.binop_scratch.float);
                        ValueT::new(tag, result_f, self.zero_ptr.clone())
                    }
                };
                res
            }
            Expr::Variable(var) => {
                let var_ptr = self.scopes.get(var).clone();
                let ptr = self.runtime_data_ptr();
                let string_tag = self.function.create_sbyte_constant(STRING_TAG as c_char);
                match expr.typ {
                    AwkT::String => {
                        let var = self.load(&var_ptr);
                        let zero = self.function.create_float64_constant(0.0);
                        let new_ptr = self.function.insn_call_native(self.runtime.copy_string(), vec![ptr, var.pointer], Some(Context::void_ptr_type()));
                        ValueT::new(string_tag, zero, new_ptr)
                    }
                    AwkT::Variable => {
                        // If it's a string variable copy it and store that pointer in self.binop_scratch.pointer
                        // otherwise store zero self.binop_scratch.pointer. After this load self.binop_scratch.pointer
                        // and make a new value with the old tag/float + new string pointer.
                        let var = self.load(&var_ptr);
                        let is_not_str = self.function.insn_eq(&string_tag, &var.tag);
                        let mut done_lbl = Label::new();
                        let mut is_not_str_lbl = Label::new();
                        self.function.insn_branch_if_not(&is_not_str, &mut is_not_str_lbl);
                        let new_ptr = self.function.insn_call_native(self.runtime.copy_string(), vec![ptr, var.pointer], Some(Context::void_ptr_type()));
                        self.function.insn_store(&self.binop_scratch.pointer, &new_ptr);
                        self.function.insn_branch(&mut done_lbl);

                        self.function.insn_label(&mut is_not_str_lbl);
                        self.function.insn_store(&self.binop_scratch.pointer, &self.zero_ptr);

                        self.function.insn_label(&mut done_lbl);
                        let str_ptr = self.function.insn_load(&self.binop_scratch.pointer);
                        ValueT::new(var.tag, var.float, str_ptr)
                    }
                    AwkT::Float => {
                        self.load(&var_ptr)
                    }
                }
            }
            Expr::Column(col) => {
                let column = self.compile_expr(col);
                let ptr = self.runtime_data_ptr();
                let val = self.function.insn_call_native(self.runtime.column(), vec![ptr, column.tag.clone(), column.float.clone(), column.pointer.clone()], Some(Context::void_ptr_type()));
                let tag = self.function.create_sbyte_constant(STRING_TAG as c_char);
                // self.drop_if_str(&column, col.typ);
                ValueT::new(tag, self.function.create_float64_constant(0.0), val)
            }
            Expr::Call => {
                // Ask runtime if there is a next line
                let one = self.function.create_sbyte_constant(FLOAT_TAG as c_char);
                let ptr = self.runtime_data_ptr();
                let next_line_exists = self.function.insn_call_native(self.runtime.next_line(), vec![ptr], Some(Context::float64_type()));
                ValueT::new(one, next_line_exists, self.zero_ptr.clone())
            }
            Expr::Concatenation(l_expr, r_expr) => {
                let left = self.compile_expr(l_expr);
                let right = self.compile_expr(r_expr);
                todo!("concat wip")
            }
        }
    }

    fn load(&mut self, ptr: &ValuePtrT) -> ValueT {
        let tag = self.function.insn_load(&ptr.tag);
        let val = self.function.insn_load(&ptr.float);
        let ptr = self.function.insn_load(&ptr.pointer);
        ValueT::new(tag, val, ptr)
    }
}