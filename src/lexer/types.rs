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

#[derive(Debug, Clone, PartialOrd, PartialEq)]
pub enum LogicalOp {
    And,
    Or,
}

#[derive(Debug, Clone, PartialOrd, PartialEq)]
pub enum Token {
    Semicolon,
    Column(usize),
    BinOp(BinOp),
    LogicalOp(LogicalOp),
    Bang,
    String(String),
    Number(f64),
    False,
    True,
    EOF,
    LeftBrace,
    RightBrace,
    LeftParen,
    RightParen,
    Print,
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
    Less,
    LessEq,
    String,
    Number,
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
    Semicolon
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
            Token::Number(_) => TokenType::Number,
            Token::False => TokenType::False,
            Token::True => TokenType::True,
            Token::EOF => TokenType::EOF,
            Token::Column(_) => TokenType::Column,
            Token::LeftBrace => TokenType::LeftBrace,
            Token::RightBrace => TokenType::RightBrace,
            Token::LeftParen => TokenType::LeftParen,
            Token::RightParen => TokenType::RightParen,
            Token::Print => TokenType::Print,
            Token::Semicolon => TokenType::Semicolon,
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
            TokenType::Number => "Number",
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
            TokenType::Semicolon => "Semicolon"
        }
    }
}