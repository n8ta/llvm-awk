use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use crate::lexer::{BinOp, LogicalOp};

#[derive(Debug, PartialEq)]
pub enum Stmt {
    Expr(Expr),
    Print(Expr),
    Assign(String, Expr),
    Return(Option<Expr>),
    Group(Vec<Stmt>),
    If(Expr, Box<Stmt>, Option<Box<Stmt>>),
}

#[derive(Debug, PartialEq)]
pub struct Block {
    pub test: Test,
    pub body: Stmt,
}

#[derive(Debug, PartialEq)]
pub enum Test {
    Expr(Expr),
    Begin,
    End,
    Always,
}

impl Block {
    pub fn new_begin(body: Stmt) -> Block { Block { test: Test::Begin, body } }
    pub fn new_end(body: Stmt) -> Block { Block { test: Test::End, body } }
    pub fn new_always(body: Stmt) -> Block { Block { test: Test::Always, body } }
    pub fn new_expr(expr: Expr, body: Stmt) -> Block { Block { test: Test::Expr(expr), body } }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    Number(f64),
    String(String),
    Ident(String),
    BinOp(Box<Expr>, BinOp, Box<Expr>),
    LogicalOp(Box<Expr>, LogicalOp, Box<Expr>),
    Variable(String),
}

impl Display for Expr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Variable(n) => write!(f, "var {}", n),
            Expr::String(str) => write!(f, "\"{}\"", str),
            Expr::Ident(str) => write!(f, "{}", str),
            Expr::Number(n) => write!(f, "{}", n),
            Expr::BinOp(left, op, right) => write!(f, "{}{}{}", left, op, right),
            Expr::LogicalOp(left, op, right) => write!(f, "{}{}{}", left, op, right),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Program {
    pub body: Vec<Block>,
}

impl Program {
    pub fn new(body: Vec<Block>) -> Program { Program { body } }
}