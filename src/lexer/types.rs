use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialOrd, PartialEq)]
pub enum BinOp {
    Minus,
    Plus,
    Slash,
    Star,
    Greater,
    GreaterEq,
    Less,
    LessEq,
    BangEq,
    EqEq,
}

impl Display for BinOp {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            BinOp::Minus => f.write_str("-"),
            BinOp::Plus => f.write_str("+"),
            BinOp::Slash => f.write_str("/"),
            BinOp::Star => f.write_str("*"),
            BinOp::Greater => f.write_str(">"),
            BinOp::GreaterEq => f.write_str(">="),
            BinOp::Less => f.write_str("<"),
            BinOp::LessEq => f.write_str("<="),
            BinOp::BangEq => f.write_str("!="),
            BinOp::EqEq => f.write_str("=="),
        }
    }
}

#[derive(Debug, Clone, PartialOrd, PartialEq)]
pub enum LogicalOp {
    And,
    Or,
}

impl Display for LogicalOp {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            LogicalOp::And => f.write_str("&&"),
            LogicalOp::Or => f.write_str("||"),
        }
    }
}

#[derive(Debug, Clone, PartialOrd, PartialEq)]
pub enum Token {
    Eq,
    Semicolon,
    Column,
    BinOp(BinOp),
    LogicalOp(LogicalOp),
    Bang,
    String(String),
    Ident(String),
    NumberF64(f64),
    False,
    True,
    EOF,
    LeftBrace,
    RightBrace,
    LeftParen,
    RightParen,
    Print,
    Ret,
    If,
    Begin,
    End,
    Else,
}

#[derive(Debug, Clone, PartialOrd, PartialEq, Hash, Eq, Ord)]
pub enum TokenType {
    Minus,
    Plus,
    Slash,
    Star,
    Bang,
    BangEq,
    EqEq,
    Greater,
    GreaterEq,
    Ident,
    Less,
    LessEq,
    String,
    NumberF64,
    And,
    Or,
    False,
    True,
    EOF,
    Column,
    LeftBrace,
    RightBrace,
    LeftParen,
    RightParen,
    Print,
    Semicolon,
    Eq,
    Ret,
    If,
    Begin,
    End,
    Else,
}

impl Token {
    pub fn ttype(&self) -> TokenType {
        // Match statement mapping every single ttype to its id
        match self {
            Token::BinOp(bin_op) => {
                match bin_op {
                    BinOp::Minus => TokenType::Minus,
                    BinOp::Plus => TokenType::Plus,
                    BinOp::Slash => TokenType::Slash,
                    BinOp::Star => TokenType::Star,
                    BinOp::Greater => TokenType::Greater,
                    BinOp::GreaterEq => TokenType::GreaterEq,
                    BinOp::Less => TokenType::Less,
                    BinOp::LessEq => TokenType::LessEq,
                    BinOp::BangEq => TokenType::BangEq,
                    BinOp::EqEq => TokenType::EqEq,
                }
            }
            Token::LogicalOp(logical_op) => {
                match logical_op {
                    LogicalOp::And => TokenType::And,
                    LogicalOp::Or => TokenType::Or,
                }
            }
            Token::Bang => TokenType::Bang,
            Token::String(_) => TokenType::String,
            Token::NumberF64(_) => TokenType::NumberF64,
            Token::False => TokenType::False,
            Token::True => TokenType::True,
            Token::EOF => TokenType::EOF,
            Token::Column => TokenType::Column,
            Token::LeftBrace => TokenType::LeftBrace,
            Token::RightBrace => TokenType::RightBrace,
            Token::LeftParen => TokenType::LeftParen,
            Token::RightParen => TokenType::RightParen,
            Token::Print => TokenType::Print,
            Token::Semicolon => TokenType::Semicolon,
            Token::Eq => TokenType::Eq,
            Token::Ret => TokenType::Ret,
            Token::If => TokenType::If,
            Token::Else => TokenType::Else,
            Token::End => TokenType::End,
            Token::Begin => TokenType::Begin,
            Token::Ident(_) => TokenType::Ident,
        }
    }
}

impl TokenType {
    pub fn name(token_type: TokenType) -> &'static str {
        match token_type {
            TokenType::Minus => "Minus",
            TokenType::Plus => "Plus",
            TokenType::Slash => "Slash",
            TokenType::Star => "Star",
            TokenType::Bang => "Bang",
            TokenType::EqEq => "EqEq",
            TokenType::Greater => "Greater",
            TokenType::GreaterEq => "GreaterEq",
            TokenType::Less => "Less",
            TokenType::LessEq => "LessEq",
            TokenType::String => "String",
            TokenType::NumberF64 => "NumberF64",
            TokenType::And => "And",
            TokenType::Or => "Or",
            TokenType::False => "False",
            TokenType::True => "True",
            TokenType::EOF => "EOF",
            TokenType::BangEq => "BangEq",
            TokenType::Column => "Column",
            TokenType::LeftBrace => "LeftBrace",
            TokenType::RightBrace => "RightBrace",
            TokenType::LeftParen => "LeftParen",
            TokenType::RightParen => "RightParen",
            TokenType::Print => "Print",
            TokenType::Semicolon => "Semicolon",
            TokenType::Eq => "Eq",
            TokenType::Ret => "Ret",
            TokenType::If => "If",
            TokenType::Else => "Else",
            TokenType::Begin => "Begin",
            TokenType::End => "End",
            TokenType::Ident => "Ident",
        }
    }
}