use crate::Expr;
use crate::parser::{Stmt};

pub fn extract(prog: &Stmt) -> Vec<String> {
    let mut vars = vec![];
    extract_stmt(prog, &mut vars);
    vars
}

fn extract_stmt(stmt: &Stmt, vars: &mut Vec<String>) {
    match stmt {
        Stmt::Expr(expr) => extract_expr(expr, vars),
        Stmt::Print(expr) => extract_expr(expr, vars),
        Stmt::Assign(var, val) => {
            extract_expr(val, vars);
            vars.push(var.clone());
        }
        Stmt::Return(expr) => if let Some(expr) = expr { extract_expr(expr, vars); },
        Stmt::Group(group) => {
            for elem in group {
                extract_stmt(elem, vars);
            }
        }
        Stmt::If(test, if_block, else_block) => {
            extract_expr(test, vars);
            extract_stmt(if_block, vars);
            if let Some(else_block) = else_block {
                extract_stmt(else_block, vars);
            }
        }
        Stmt::While(test, body) => {
            extract_expr(test, vars);
            extract_stmt(body, vars);
        }
    }
}

fn extract_expr(expr: &Expr, vars: &mut Vec<String>) {
    match expr {
        Expr::Variable(var) => vars.push(var.clone()),
        Expr::String(_str) => {}
        Expr::NumberF64(n) => {}
        Expr::BinOp(left, op, right) => {
            extract_expr(left, vars);
            extract_expr(right, vars);
        }
        Expr::LogicalOp(left, op, right) => {
            extract_expr(left, vars);
            extract_expr(right, vars);
        }
        Expr::Column(col) => extract_expr(col, vars),
        Expr::Call => {}
    }
}