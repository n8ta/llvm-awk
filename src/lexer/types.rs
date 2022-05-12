#[derive(Debug, Clone, PartialOrd, PartialEq)]
pub enum Token {
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
    String(String),
    Number(f64),
    And,
    Or,
    False,
    True,
    EOF,
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
}

impl Token {
    pub fn ttype(&self) -> TokenType {
        // Match statement mapping every single ttype to its id
        match self {
            Token::Minus => TokenType::Minus,
            Token::Plus => TokenType::Plus,
            Token::Slash => TokenType::Slash,
            Token::Star => TokenType::Star,
            Token::Bang => TokenType::Bang,
            Token::EqEq => TokenType::EqEq,
            Token::Greater => TokenType::Greater,
            Token::GreaterEq => TokenType::GreaterEq,
            Token::Less => TokenType::Less,
            Token::LessEq => TokenType::LessEq,
            Token::String(_) => TokenType::String,
            Token::Number(_) => TokenType::Number,
            Token::And => TokenType::And,
            Token::Or => TokenType::Or,
            Token::False => TokenType::False,
            Token::True => TokenType::True,
            Token::EOF => TokenType::EOF,
            Token::BangEq => TokenType::BangEq,
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
        }
    }
}