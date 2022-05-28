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
pub struct PatternAction {
    pub pattern: Option<Expr>,
    pub action: Stmt
}
impl PatternAction {
    pub fn new(pattern: Option<Expr>, action: Stmt) -> Self {
        Self { pattern, action }
    }
    pub fn new_pattern_only(test: Expr) -> PatternAction { PatternAction::new(Some(test), Stmt::Print(Expr::Column(Box::new(Expr::Number(0.0))))) }
    pub fn new_action_only(body: Stmt) -> PatternAction { PatternAction::new(None, body) }
}


#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    Number(f64),
    String(String),
    BinOp(Box<Expr>, BinOp, Box<Expr>),
    LogicalOp(Box<Expr>, LogicalOp, Box<Expr>),
    Variable(String),
    Column(Box<Expr>),
}

impl Display for Expr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Variable(n) => write!(f, "var {}", n),
            Expr::String(str) => write!(f, "\"{}\"", str),
            Expr::Number(n) => write!(f, "{}", n),
            Expr::BinOp(left, op, right) => write!(f, "{}{}{}", left, op, right),
            Expr::LogicalOp(left, op, right) => write!(f, "{}{}{}", left, op, right),
            Expr::Column(col) => write!(f, "{}", col),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Program {
    pub begins: Vec<Stmt>,
    pub ends: Vec<Stmt>,
    pub pattern_actions: Vec<PatternAction>,
}

impl Program {
    pub fn new(begins: Vec<Stmt>, ends: Vec<Stmt>, pattern_actions: Vec<PatternAction>) -> Program { Program { begins, ends, pattern_actions } }
    pub fn new_action_only(stmt: Stmt) -> Program { Program { begins: vec![], ends: vec![], pattern_actions: vec![PatternAction::new_action_only(stmt)] } }
}