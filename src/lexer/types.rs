use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialOrd, PartialEq, Copy)]
pub enum MathOp {
    Minus,
    Plus,
    Slash,
    Star,
}

#[derive(Debug, Clone, PartialOrd, PartialEq, Copy)]
pub enum BinOp {
    Greater,
    GreaterEq,
    Less,
    LessEq,
    BangEq,
    EqEq,
    MatchedBy,
    NotMatchedBy,
}

// impl BinOp {
//     pub fn predicate(&self) -> FloatPredicate {
//         match self {
//             BinOp::Greater => FloatPredicate::OGT,
//             BinOp::GreaterEq => FloatPredicate::OGE,
//             BinOp::Less => FloatPredicate::OLT,
//             BinOp::LessEq => FloatPredicate::OLE,
//             BinOp::BangEq => FloatPredicate::ONE,
//             BinOp::EqEq => FloatPredicate::OEQ,
//             _ => panic!("Unsupported binop"),
//         }
//     }
// }

impl Display for BinOp {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            BinOp::Greater => f.write_str(">"),
            BinOp::GreaterEq => f.write_str(">="),
            BinOp::Less => f.write_str("<"),
            BinOp::LessEq => f.write_str("<="),
            BinOp::BangEq => f.write_str("!="),
            BinOp::EqEq => f.write_str("=="),
            BinOp::MatchedBy => f.write_str("~"),
            BinOp::NotMatchedBy => f.write_str("!~"),
        }
    }
}

impl Display for MathOp {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            MathOp::Minus => f.write_str("-"),
            MathOp::Plus => f.write_str("+"),
            MathOp::Slash => f.write_str("/"),
            MathOp::Star => f.write_str("*"),
        }
    }
}

#[derive(Debug, Clone, PartialOrd, PartialEq, Copy)]
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
    MathOp(MathOp),
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
    While,
    For,
    Do,
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
    For,
    While,
    Do,
    MatchedBy,
    NotMatchedBy,
}

impl Token {
    pub fn ttype(&self) -> TokenType {
        // Match statement mapping every single ttype to its id
        match self {
            Token::BinOp(bin_op) => {
                match bin_op {
                    BinOp::Greater => TokenType::Greater,
                    BinOp::GreaterEq => TokenType::GreaterEq,
                    BinOp::Less => TokenType::Less,
                    BinOp::LessEq => TokenType::LessEq,
                    BinOp::BangEq => TokenType::BangEq,
                    BinOp::EqEq => TokenType::EqEq,
                    BinOp::MatchedBy => TokenType::MatchedBy,
                    BinOp::NotMatchedBy => TokenType::NotMatchedBy,
                }
            }
            Token::MathOp(math_op) => {
                match math_op {
                    MathOp::Minus => TokenType::Minus,
                    MathOp::Plus => TokenType::Plus,
                    MathOp::Slash => TokenType::Slash,
                    MathOp::Star => TokenType::Star,
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
            Token::While => TokenType::While,
            Token::For => TokenType::For,
            Token::Do => TokenType::Do,
        }
    }
}

impl TokenType {
    pub fn name(token_type: TokenType) -> &'static str {
        match token_type {
            TokenType::While => "While",
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
            TokenType::For => "For",
            TokenType::Do => "Do",
            TokenType::MatchedBy => "~",
            TokenType::NotMatchedBy => "~!",
        }
    }
}