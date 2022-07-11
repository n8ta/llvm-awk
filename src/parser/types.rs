use std::fmt::{Display, Formatter};
use crate::lexer::{BinOp, MathOp, LogicalOp};


#[derive(PartialEq, Clone, Copy, Debug)]
pub enum AwkT {
    String,
    Float,
    Variable,
}

#[derive(Debug, PartialEq)]
pub enum Stmt {
    Expr(TypedExpr),
    Print(TypedExpr),
    Group(Vec<Stmt>),
    If(TypedExpr, Box<Stmt>, Option<Box<Stmt>>),
    While(TypedExpr, Box<Stmt>),
}

impl Display for Stmt {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Stmt::Expr(expr) => { write!(f, "{}", expr)? },
            Stmt::Print(expr) => { write!(f, "print {}", expr)? },
            Stmt::Group(group) => {
                for elem in group {
                    write!(f, "{}", elem)?;
                }
            }
            Stmt::If(test, if_so, if_not) => {
                write!(f, "if {} {{{}}}", test, if_so)?;
                if let Some(else_case) = if_not {
                    write!(f, "else {{ {} }}", else_case)?;
                }
            }
            Stmt::While(test, body) => {
                write!(f, "while {} {{{}}} ", test, body)?;
            }
        };
        write!(f, "\n")
    }
}

#[derive(Debug, PartialEq)]
pub struct PatternAction {
    pub pattern: Option<TypedExpr>,
    pub action: Stmt,
}

impl PatternAction {
    pub fn new(pattern: Option<TypedExpr>, action: Stmt) -> Self {
        Self { pattern, action }
    }
    pub fn new_pattern_only(test: TypedExpr) -> PatternAction { PatternAction::new(Some(test), Stmt::Print(TypedExpr::new_str(Expr::Column(Box::new(TypedExpr::new_num(Expr::NumberF64(0.0))))))) }
    pub fn new_action_only(body: Stmt) -> PatternAction { PatternAction::new(None, body) }
}


#[derive(Debug, PartialEq, Clone)]
pub struct TypedExpr {
    pub typ: AwkT,
    pub expr: Expr,
}

impl TypedExpr {
    pub fn new_num(expr: Expr) -> TypedExpr { TypedExpr { typ: AwkT::Float, expr } }
    pub fn new_str(expr: Expr) -> TypedExpr { TypedExpr { typ: AwkT::String, expr } }
    pub fn new_var(expr: Expr) -> TypedExpr { TypedExpr { typ: AwkT::Variable, expr } }
    pub fn new(typ: AwkT, expr: Expr) -> TypedExpr { TypedExpr { typ, expr } }
}

impl Into<TypedExpr> for Expr {
    fn into(self) -> TypedExpr {
        TypedExpr::new_var(self)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    Assign(String, Box<TypedExpr>),
    NumberF64(f64),
    String(String),
    BinOp(Box<TypedExpr>, BinOp, Box<TypedExpr>),
    MathOp(Box<TypedExpr>, MathOp, Box<TypedExpr>),
    LogicalOp(Box<TypedExpr>, LogicalOp, Box<TypedExpr>),
    Variable(String),
    Column(Box<TypedExpr>),
    Call,
}


impl Display for TypedExpr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.typ {
            AwkT::String => write!(f, "(s {})", self.expr),
            AwkT::Float => write!(f, "(f {})", self.expr),
            AwkT::Variable => write!(f, "(v {})", self.expr),
        }
    }
}

impl Display for Expr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Assign(var, expr) => write!(f, "{} = {}", var, expr),
            Expr::Call => write!(f, "check_if_there_is_another_line"),
            Expr::Variable(n) => write!(f, "{}", n),
            Expr::String(str) => write!(f, "\"{}\"", str),
            Expr::NumberF64(n) => write!(f, "{}", n),
            Expr::BinOp(left, op, right) => write!(f, "{}{}{}", left, op, right),
            Expr::MathOp(left, op, right) => write!(f, "{}{}{}", left, op, right),
            Expr::LogicalOp(left, op, right) => write!(f, "{}{}{}", left, op, right),
            Expr::Column(col) => write!(f, "${}", col),
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
    #[allow(dead_code)]
    pub fn new_action_only(stmt: Stmt) -> Program { Program { begins: vec![], ends: vec![], pattern_actions: vec![PatternAction::new_action_only(stmt)] } }
}