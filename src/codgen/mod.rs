mod scopes;
// mod runtime;
// mod subroutines;
mod variable_extract;

use std::os::raw::{c_char, c_double, c_void};
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
    binop_scratch: ValuePtrT, // Since we don't have phis just store the result of binops here
}

type ValueT = (Value, Value);
type ValuePtrT = ValueT;

impl CodeGen {
    fn new(files: Vec<String>, capture: bool) -> Self {
        let mut context = Context::new();
        let mut function = context.function(Abi::Cdecl, Context::float64_type(), vec![]).expect("to create function");
        let runtime = Runtime::new(files, capture);
        let binop_scratch_tag = function.create_value_int();
        let binop_scratch_value = function.create_value_float64();
        let binop_scratch = (binop_scratch_tag, binop_scratch_value);
        let codegen = CodeGen {
            function,
            scopes: Scopes::new(),
            context,
            runtime,
            binop_scratch,
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
    }

    fn runtime_data_ptr(&mut self) -> Value {
        self.function.create_void_ptr_constant(self.runtime.data_ptr())
    }

    fn create_value(&mut self) -> ValuePtrT {
        let tag = self.function.create_value_int();
        let value = self.function.create_value_float64();
        let zero = self.function.create_float64_constant(0 as c_double);
        let zero_tag = self.function.create_sbyte_constant(0);
        self.function.insn_store(&tag, &zero_tag);
        self.function.insn_store(&value, &zero);
        (tag, value)
    }

    fn define_all_vars(&mut self, prog: &Stmt) {
        let vars = variable_extract::extract(prog);

        for var in vars {
            let val = self.create_value();
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
                let test = self.compile_expr(test);
                let ptr = self.runtime_data_ptr();
                let bool_value = self.function.insn_call_native(self.runtime.is_truthy, vec![ptr, test.0, test.1], Some(Context::int_type()));
                self.function.insn_branch_if_not(&bool_value, &mut done_label);
                self.compile_stmt(body);
                self.function.insn_branch(&mut test_label);
                self.function.insn_label(&mut done_label);
            }
        }
    }

    fn to_float(&mut self, value: ValueT) -> Value {
        let zero = self.function.create_sbyte_constant(FLOAT_TAG as c_char);
        let one = self.function.create_sbyte_constant(STRING_TAG as c_char);
        let (tag, value) = value;
        let mut done_lbl = Label::new();
        self.function.insn_store(&self.binop_scratch.1, &value);
        let is_float = self.function.insn_eq(&tag, &zero);
        self.function.insn_branch_if(&is_float, &mut done_lbl);

        let ptr = self.runtime_data_ptr();
        let res = self.function.insn_call_native(self.runtime.string_to_number, vec![ptr, tag, value], Some(Context::float64_type()));
        self.function.insn_store(&self.binop_scratch.1, &res);

        self.function.insn_label(&mut done_lbl);
        self.function.insn_load(&self.binop_scratch.1)
    }

    fn compile_expr(&mut self, expr: &TypedExpr) -> ValueT {
        match &expr.expr {
            Expr::NumberF64(num) => {
                let res = (self.function.create_sbyte_constant(FLOAT_TAG as c_char),
                           self.function.create_float64_constant(*num));
                res
            }
            Expr::String(str) => {
                let boxed = Box::new(str.to_string());
                let tag = self.function.create_sbyte_constant(STRING_TAG as c_char);
                let raw_ptr = Box::into_raw(boxed);
                let ptr = self.function.create_void_ptr_constant(raw_ptr as *mut c_void);
                (tag, ptr)
            }
            Expr::MathOp(left_expr, op, right_expr) => {
                let mut left = self.compile_expr(left_expr);
                let mut right = self.compile_expr(right_expr);
                let zero = self.function.create_sbyte_constant(FLOAT_TAG as c_char);

                if AwkT::Float != left_expr.typ {
                    left = (zero.clone(), self.to_float(left));
                }
                if AwkT::Float != right_expr.typ {
                    right = (zero.clone(), self.to_float(right));
                }

                let res = match op {
                    MathOp::Minus => {
                        self.function.insn_sub(&left.1, &right.1)
                    }
                    MathOp::Plus => {
                        self.function.insn_add(&left.1, &right.1)
                    }
                    MathOp::Slash => {
                        self.function.insn_div(&left.1, &right.1)
                    }
                    MathOp::Star => {
                        self.function.insn_mult(&left.1, &right.1)
                    }
                };
                (zero, res)
            }
            Expr::BinOp(left_expr, op, right_expr) => {
                let left = self.compile_expr(left_expr);
                let right = self.compile_expr(right_expr);
                let tag = self.function.create_sbyte_constant(FLOAT_TAG as c_char);
                let value = match (left_expr.typ, right_expr.typ) {
                    (AwkT::Float, AwkT::Float) => {
                        match op {
                            BinOp::Greater => self.function.insn_gt(&left.1, &right.1),
                            BinOp::GreaterEq => self.function.insn_ge(&left.1, &right.1),
                            BinOp::Less => self.function.insn_lt(&left.1, &right.1),
                            BinOp::LessEq => self.function.insn_le(&left.1, &right.1),
                            BinOp::BangEq => self.function.insn_ne(&left.1, &right.1),
                            BinOp::EqEq => self.function.insn_eq(&left.1, &right.1),
                            BinOp::MatchedBy => todo!("matched expr"),
                            BinOp::NotMatchedBy => todo!("matched expr"),
                        }
                    }
                    _ => {
                        todo!("non float float binop ")
                    }
                };
                (tag, value)
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
                let tag = self.function.create_sbyte_constant(STRING_TAG as c_char);
                (tag, val)
            }
            Expr::Call => {
                let one = self.function.create_sbyte_constant(FLOAT_TAG as c_char);
                let ptr = self.runtime_data_ptr();
                let next_line_exists = self.function.insn_call_native(self.runtime.next_line, vec![ptr], Some(Context::float64_type()));
                (one, next_line_exists)
            }
        }
    }
}