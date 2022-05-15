use crate::lexer::BinOp;

pub enum Stmt {
    Expr(Expr),
    Print(Expr),
}

pub enum Expr {
    Number(f64),
    BinOp(Box<Expr>, BinOp, Box<Expr>),
}
struct Program {
    body: Vec<(Option<Expr>, Stmt)>,
}