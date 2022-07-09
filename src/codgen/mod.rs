mod scopes;
// mod runtime;
// mod subroutines;
mod variable_extract;

use std::mem::size_of;
use std::os::raw::{c_char, c_double, c_long, c_void};
use gnu_libjit::{Abi, Context, Function, Label, Value};
use crate::{Expr};
use crate::codgen::scopes::Scopes;
use crate::lexer::{BinOp, MathOp};
use crate::parser::{AwkT, Stmt, TypedExpr};
use crate::runtime::{Runtime};

/// Value type
///
/// tag: u8   (0 is f64, 2 is string)
/// | number f64
/// | string *mut String

pub fn compile_and_run(prog: Stmt, files: &[String], dump: bool) {
    let mut codegen = CodeGen::new(files.to_vec(), false);
    codegen.compile(prog, files, dump);
    codegen.run()
}

pub fn compile_and_capture(prog: Stmt, files: &[String], dump: bool) -> String {
    let mut codegen = CodeGen::new(files.to_vec(), true);
    codegen.compile(prog, files, dump);
    codegen.run();
    codegen.runtime.output()
}

pub const FLOAT_TAG: u8 = 0;
pub const STRING_TAG: u8 = 1;

struct CodeGen {
    function: Function,
    scopes: Scopes,
    context: Context,
    runtime: Runtime,
    binop_scratch: ValuePtrT,
    // Since we don't have phis just store the result of binops here
    binop_scratch_int: Value,
    zero_ptr: Value, // Used to init the pointer section of the value struct
}

#[derive(Clone)]
struct ValueT {
    pub tag: Value,
    pub float: Value,
    pub pointer: Value,
}

impl ValueT {
    pub fn new(tag: Value, float: Value, pointer: Value) -> ValueT { ValueT { tag, float, pointer } }
}

type ValuePtrT = ValueT;

impl CodeGen {
    fn new(files: Vec<String>, capture: bool) -> Self {
        let mut context = Context::new();
        let mut function = context.function(Abi::Cdecl, Context::float64_type(), vec![]).expect("to create function");
        let runtime = Runtime::new(files, capture);
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
        // if dump {
        //     println!("{}", self.function.dump().unwrap());
        // }
    }

    fn runtime_data_ptr(&mut self) -> Value {
        self.function.create_void_ptr_constant(self.runtime.data_ptr())
    }

    fn create_value(&mut self) -> ValuePtrT {
        let tag = self.function.create_value_int();
        let value = self.function.create_value_float64();
        let ptr = self.function.create_value_void_ptr();

        let zero = self.function.create_float64_constant(0 as c_double);
        let zero_tag = self.function.create_sbyte_constant(0);


        self.function.insn_store(&tag, &zero_tag);
        self.function.insn_store(&value, &zero);
        self.function.insn_store(&ptr, &self.zero_ptr.clone());
        ValueT::new(tag, value, ptr)
    }

    fn define_all_vars(&mut self, prog: &Stmt) {
        let vars = variable_extract::extract(prog);

        for var in vars {
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

    fn compile_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Expr(expr) => { self.compile_expr(expr); }
            Stmt::Print(expr) => {
                let val = self.compile_expr(expr);
                let ptr = self.runtime_data_ptr();
                match expr.typ {
                    AwkT::String => {
                        self.function.insn_call_native(self.runtime.print_string, vec![ptr, val.pointer], None);
                    }
                    AwkT::Float => {
                        self.function.insn_call_native(self.runtime.print_float, vec![ptr, val.float], None);
                    }
                    AwkT::Variable => {
                        let zero_tag = self.function.create_sbyte_constant(FLOAT_TAG as c_char);
                        let mut float_lbl = Label::new();
                        let mut done_lbl = Label::new();
                        let tag_is_zero = self.function.insn_eq(&zero_tag, &val.tag);
                        self.function.insn_branch_if(&tag_is_zero, &mut float_lbl);
                        self.function.insn_call_native(self.runtime.print_string, vec![ptr.clone(), val.pointer], None);
                        self.function.insn_branch(&mut done_lbl);
                        self.function.insn_label(&mut float_lbl);
                        self.function.insn_call_native(self.runtime.print_float, vec![ptr, val.float], None);
                        self.function.insn_label(&mut done_lbl);
                    }
                }
            }
            Stmt::Assign(variable, expr) => {
                let val = self.compile_expr(expr);
                let variable_ptr = self.scopes.get(variable);
                self.function.insn_store(&variable_ptr.tag, &val.tag);
                self.function.insn_store(&variable_ptr.float, &val.float);
                self.function.insn_store(&variable_ptr.pointer, &val.pointer);
            }
            Stmt::Group(group) => {
                for group in group {
                    self.compile_stmt(group)
                }
            }
            Stmt::If(test, if_so, if_not) => {
                let test_value = self.compile_expr(test);
                let ptr = self.runtime_data_ptr();

                // let bool_value = self.function.insn_call_native(self.runtime.is_truthy, vec![ptr, test.0, test.1], Some(Context::int_type()));
                let bool_value = self.truthy_ret_integer(&test_value, test.typ);
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
                let ptr = self.runtime_data_ptr();
                let bool_value = self.truthy_ret_integer(&test_value, test.typ);
                self.function.insn_branch_if_not(&bool_value, &mut done_label);
                self.compile_stmt(body);
                self.function.insn_branch(&mut test_label);
                self.function.insn_label(&mut done_label);
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
        let res = self.function.insn_call_native(self.runtime.string_to_number, vec![ptr, value.tag, value.pointer], Some(Context::float64_type()));
        self.function.insn_store(&self.binop_scratch.float, &res);

        self.function.insn_label(&mut done_lbl);
        self.function.insn_load(&self.binop_scratch.float)
    }

    fn compile_expr(&mut self, expr: &TypedExpr) -> ValueT {
        match &expr.expr {
            Expr::NumberF64(num) =>
                ValueT::new(
                    self.function.create_sbyte_constant(FLOAT_TAG as c_char),
                    self.function.create_float64_constant(*num),
                    self.zero_ptr.clone()),
            Expr::String(str) => {
                let boxed = Box::new(str.to_string());
                let tag = self.function.create_sbyte_constant(STRING_TAG as c_char);
                let raw_ptr = unsafe { Box::into_raw(boxed) };
                // println!("rawptr {:?}", raw_ptr);
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
                    MathOp::Minus => {
                        self.function.insn_sub(&left.float, &right.float)
                    }
                    MathOp::Plus => {
                        self.function.insn_add(&left.float, &right.float)
                    }
                    MathOp::Slash => {
                        self.function.insn_div(&left.float, &right.float)
                    }
                    MathOp::Star => {
                        self.function.insn_mult(&left.float, &right.float)
                    }
                };
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

                ValueT::new(tag, self.function.insn_load(&self.binop_scratch.float), self.zero_ptr.clone())
            }
            Expr::LogicalOp(left, op, right) => {
                todo!("logical op")
            }
            Expr::Variable(var) => {
                let var_ptr = self.scopes.get(var);
                let tag = self.function.insn_load(&var_ptr.tag);
                let val = self.function.insn_load(&var_ptr.float);
                let ptr = self.function.insn_load(&var_ptr.pointer);
                ValueT::new(tag, val, ptr)
            }
            Expr::Column(col) => {
                let column = self.compile_expr(col);
                let ptr = self.runtime_data_ptr();
                let val = self.function.insn_call_native(
                    self.runtime.column,
                    vec![ptr, column.tag, column.float, column.pointer],
                    Some(Context::void_ptr_type()));
                let tag = self.function.create_sbyte_constant(STRING_TAG as c_char);
                ValueT::new(tag,  self.function.create_float64_constant(0.0), val)
            }
            Expr::Call => {
                let one = self.function.create_sbyte_constant(FLOAT_TAG as c_char);
                let ptr = self.runtime_data_ptr();
                let next_line_exists = self.function.insn_call_native(self.runtime.next_line, vec![ptr], Some(Context::float64_type()));
                ValueT::new(one, next_line_exists, self.zero_ptr.clone())
            }
        }
    }
}