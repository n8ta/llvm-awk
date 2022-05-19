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

#[derive(Debug, PartialEq)]
pub enum Expr {
    Number(f64),
    BinOp(Box<Expr>, BinOp, Box<Expr>),
    LogicalOp(Box<Expr>, LogicalOp, Box<Expr>),
}

#[derive(Debug, PartialEq)]
pub struct Program {
    body: Vec<Block>,
}

impl Program {
    pub fn new(body: Vec<Block>) -> Program { Program { body } }
}