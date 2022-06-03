// use crate::Expr;
// use crate::parser::{Program, Stmt};
//
// pub fn extract(prog: &Program) -> Vec<String> {
//     let mut vars = vec![];
//     for begin in prog.begins.iter() {
//         extract_stmt(begin, &mut vars);
//     }
//     for end in prog.ends.iter() {
//         extract_stmt(end, &mut vars);
//     }
//     for pat_act in prog.pattern_actions.iter() {
//         extract_stmt(&pat_act.action, &mut vars);
//     }
//     vars
// }
//
// fn extract_stmt(stmt: &Stmt, vars: &mut Vec<String>) {
//     match stmt {
//         Stmt::Expr(expr) => extract_expr(expr, vars),
//         Stmt::Print(expr) => extract_expr(expr, vars),
//         Stmt::Assign(var, val) => {
//             extract_expr(val, vars);
//             vars.push(var.clone());
//         }
//         Stmt::Return(expr) => if let Some(expr) = expr { extract_expr(expr, vars); },
//         Stmt::Group(group) => {
//             for elem in group {
//                 extract_stmt(elem, vars);
//             }
//         }
//         Stmt::If(test, if_block, else_block) => {
//             extract_expr(test, vars);
//             extract_stmt(if_block, vars);
//             if let Some(else_block) = else_block {
//                 extract_stmt(else_block, vars);
//             }
//         }
//     }
// }
//
// fn extract_expr(expr: &Expr, vars: &mut Vec<String>) {
//     match expr {
//         Expr::Variable(var) => vars.push(var.clone()),
//         Expr::String(str) => {}
//         Expr::NumberF64(n) => {}
//         Expr::NumberI64(n) => {}
//         Expr::BinOp(left, op, right) => {
//             extract_expr(left, vars);
//             extract_expr(right, vars);
//         }
//         Expr::LogicalOp(left, op, right) => {
//             extract_expr(left, vars);
//             extract_expr(right, vars);
//         }
//         Expr::Column(col) => extract_expr(col, vars),
//     }
// }