use std::fmt::{Display, Formatter};
use crate::lexer::{BinOp, LogicalOp};

#[derive(Debug, PartialEq)]
pub enum Stmt {
    Expr(Expr),
    Print(Expr),
}

#[derive(Debug, PartialEq)]
pub struct Block {
    test: Option<Expr>,
    body: Vec<Stmt>,
}

impl Block {
    pub fn new(test: Option<Expr>, body: Vec<Stmt>) -> Block { Block { test, body } }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    Number(f64),
    BinOp(Box<Expr>, BinOp, Box<Expr>),
    LogicalOp(Box<Expr>, LogicalOp, Box<Expr>),
}

impl Display for Expr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Number(n) => write!(f, "{}", n),
            Expr::BinOp(left, op, right) => write!(f, "{}{}{}", left, op, right),
            Expr::LogicalOp(left, op, right) => write!(f, "{}{}{}", left, op, right),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Program {
    body: Vec<Block>,
}

impl Program {
    pub fn new(body: Vec<Block>) -> Program { Program { body } }
    pub fn expr(&self) -> Expr {
        let mut expr = &self.body[0].body[0];
        if let Stmt::Expr(expr) = expr {
            (*expr).clone()
        } else {
            panic!("Expected an expression");
        }
    }
}