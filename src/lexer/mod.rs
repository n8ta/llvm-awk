mod types;

pub use types::{Token, TokenType, BinOp, LogicalOp};

pub fn lex(str: &str) -> LexerResult {
    let mut lexer = Lexer::new(str);
    lexer.scan_tokens()?;
    Ok(lexer.tokens)
}


struct Lexer<'a> {
    src: &'a str,
    start: usize,
    current: usize,
    line: usize,
    tokens: Vec<Token>,
}

type LexerResult = Result<Vec<Token>, (String, usize)>;


impl<'a> Lexer<'a> {
    fn new(src: &'a str) -> Lexer {
        Lexer {
            src,
            start: 0,
            current: 0,
            line: 0,
            tokens: vec![],
        }
    }
    fn is_at_end(&self) -> bool {
        self.current >= self.src.chars().count()
    }
    fn advance(&mut self) -> char {
        let x = self.src.chars().nth(self.current).unwrap();
        self.current += 1;
        x
    }
    fn add_token(&mut self, tt: Token) {
        self.tokens.push(tt);
    }
    fn string(&mut self) -> Result<(), String> {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' { self.line += 1; }
            self.advance();
        }
        if self.is_at_end() {
            let partial_str = self.src.chars().skip(self.start).take(self.src.len() - self.start).collect::<String>();
            return Err(format!("Unterminated String: {}", partial_str));
        }
        self.advance();
        let str = self.src.chars().skip(self.start + 1).take(self.current - self.start - 2).collect::<String>();
        self.add_token(Token::String(str));
        return Ok(());
    }
    fn number(&mut self) -> Result<Token, String> {
        let mut is_float = false;
        while self.peek().is_digit(10) { self.advance(); }
        if self.peek() == '.' && self.peek_next().is_digit(10) {
            is_float = true;
            self.advance();
        }
        while self.peek().is_digit(10) { self.advance(); }

        let num = self.src.chars().skip(self.start).take(self.current - self.start).collect::<String>();
        // TODO: scientific notation
        match num.parse::<f64>() {
            Ok(float) => Ok(Token::NumberF64(float)),
            Err(_) => {
                return Err(format!("Unable to parse f64 {}", num));
            }
        }
    }
    #[allow(dead_code)]
    fn number_usize(&mut self, skip: usize) -> Result<usize, String> {
        while self.peek().is_digit(10) { self.advance(); }
        let num = self.src.chars().skip(self.start + skip).take(self.current - (self.start + skip)).collect::<String>();
        let usize = match num.parse::<usize>() {
            Ok(float) => float,
            Err(_) => {
                return Err(format!("Unable to parse usize{}", num));
            }
        };
        if self.peek() == '.' {
            return Err(String::from("Cannot have a decimal after a non-float number"));
        }
        Ok(usize)
    }
    fn identifier(&mut self) -> Result<(), String> {
        while self.peek().is_alphanumeric() { self.advance(); }
        let src: String = self.src.chars().skip(self.start).take(self.current - self.start).collect();
        let src = src.to_ascii_lowercase();
        if src == "true" {
            self.add_token(Token::True);
        } else if src == "false" {
            self.add_token(Token::False);
        } else if src == "return" {
            self.add_token(Token::Ret);
        } else if src == "if" {
            self.add_token(Token::If);
        } else if src == "else" {
            self.add_token(Token::Else);
        } else if src == "begin" {
            self.add_token(Token::Begin);
        } else if src == "end" {
            self.add_token(Token::End);
        } else if src == "print" {
            self.add_token(Token::Print);
        } else {
            self.add_token(Token::Ident(src));
        }

        // }
        // };
        Ok(())
    }
    fn peek(&mut self) -> char {
        match self.src.chars().nth(self.current) {
            None => 0x0 as char,
            Some(c) => c,
        }
    }
    fn peek_next(&self) -> char {
        match self.src.chars().nth(self.current + 1) {
            None => 0x0 as char,
            Some(c) => c,
        }
    }
    fn scan_token(&mut self) -> Result<(), String> {
        let c = self.advance();
        match c {
            '$' => self.add_token(Token::Column),
            // '(' => self.add_token(Token::LeftParen),
            // ')' => self.add_token(Token::RightParen),
            // '{' => self.add_token(Token::LeftBrace),
            // '}' => self.add_token(Token::RightBrace),
            '-' => self.add_token(Token::BinOp(BinOp::Minus)),
            '+' => self.add_token(Token::BinOp(BinOp::Plus)),
            // ';' => self.add_token(Token::Semicolon),
            '*' => self.add_token(Token::BinOp(BinOp::Star)),
            '!' => {
                let tt = match self.matches('=') {
                    true => Token::BinOp(BinOp::BangEq),
                    false => Token::Bang,
                };
                self.add_token(tt);
            }
            '|' => {
                let tt = match self.matches('|') {
                    true => Token::LogicalOp(LogicalOp::Or),
                    false => return Err("| must be followed by ||".to_string()),
                };
                self.add_token(tt);
            }
            '&' => {
                let tt = match self.matches('&') {
                    true => Token::LogicalOp(LogicalOp::And),
                    false => return Err("| must be followed by &".to_string()),
                };
                self.add_token(tt);
            }
            '=' => {
                let tt = match self.matches('=') {
                    true => Token::BinOp(BinOp::EqEq),
                    false => Token::Eq,
                };
                self.add_token(tt)
            }
            '<' => {
                let tt = match self.matches('=') {
                    true => Token::BinOp(BinOp::LessEq),
                    false => Token::BinOp(BinOp::Less)
                };
                self.add_token(tt)
            }
            '>' => {
                let tt = match self.matches('=') {
                    true => Token::BinOp(BinOp::GreaterEq),
                    false => Token::BinOp(BinOp::Greater)
                };
                self.add_token(tt)
            }
            '/' => {
                if self.matches('/') {
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else {
                    self.add_token(Token::BinOp(BinOp::Slash));
                }
            }
            '{' => self.add_token(Token::LeftBrace),
            '}' => self.add_token(Token::RightBrace),
            '(' => self.add_token(Token::LeftParen),
            ')' => self.add_token(Token::RightParen),
            ';' => self.add_token(Token::Semicolon),
            '"' => self.string()?,
            '\r' => (),
            '\t' => (),
            ' ' => (),
            '\n' => self.line += 1,
            _ => {
                if c.is_digit(10) {
                    let num = self.number()?;
                    self.add_token(num);
                } else if c.is_alphabetic() {
                    self.identifier()?;
                } else {
                    return Err(format!("Unexpected token::: `{}`", c));
                }
            }
        }
        Ok(())
    }

    fn matches(&mut self, expected: char) -> bool {
        if self.is_at_end() { return false; }
        if self.src.chars().nth(self.current).unwrap() != expected { return false; }
        self.current += 1;
        true
    }

    fn scan_tokens(&mut self) -> LexerResult {
        while !self.is_at_end() {
            if let Err(x) = self.scan_token() {
                return Err((x, self.line));
            }
            self.start = self.current;
        }
        self.tokens.push(Token::EOF);
        // self.tokens.push(Token::new_src(
        //     Token::EOF,
        //     self.current,
        //     self.current - self.start,
        //     self.line,
        //     self.source.clone(),
        // ));
        Ok(self.tokens.clone())
    }
}

#[test]
fn test_braces() {
    assert_eq!(lex("{ } ( ) (( )) {{ }}").unwrap(),
               vec![Token::LeftBrace, Token::RightBrace, Token::LeftParen, Token::RightParen, Token::LeftParen, Token::LeftParen, Token::RightParen, Token::RightParen, Token::LeftBrace, Token::LeftBrace, Token::RightBrace, Token::RightBrace, Token::EOF]);
}

#[test]
fn test_column_simple() {
    let str = "$1";
    let tokens = lex(str).unwrap();
    assert_eq!(tokens, vec![Token::Column, Token::NumberF64(1.0), Token::EOF]);
}


#[test]
fn test_columns() {
    let str = "$1 + $2000 $0";
    let tokens = lex(str).unwrap();
    assert_eq!(tokens, vec![Token::Column, Token::NumberF64(1.0), Token::BinOp(BinOp::Plus), Token::Column, Token::NumberF64(2000.0), Token::Column, Token::NumberF64(0.0), Token::EOF]);
}

#[test]
fn test_lex_binops_and_true_false() {
    let str = "4*2+1-2+false/true";
    let tokens = lex(str).unwrap();
    assert_eq!(tokens, vec![Token::NumberF64(4.0), Token::BinOp(BinOp::Star), Token::NumberF64(2.0), Token::BinOp(BinOp::Plus), Token::NumberF64(1.0), Token::BinOp(BinOp::Minus), Token::NumberF64(2.0), Token::BinOp(BinOp::Plus), Token::False, Token::BinOp(BinOp::Slash), Token::True, Token::EOF]);
}

#[test]
fn test_lex_decimals() {
    let str = "4.123-123.123";
    assert_eq!(lex(str).unwrap(), vec![Token::NumberF64(4.123), Token::BinOp(BinOp::Minus), Token::NumberF64(123.123), Token::EOF]);
}

#[test]
fn test_lex_equality() {
    let str = "4 != 5 == 6";
    assert_eq!(lex(str).unwrap(), vec![Token::NumberF64(4.0), Token::BinOp(BinOp::BangEq), Token::NumberF64(5.0), Token::BinOp(BinOp::EqEq), Token::NumberF64(6.0), Token::EOF]);
}

#[test]
fn test_lex_logical_op() {
    let str = "4 && 5 || 6";
    assert_eq!(lex(str).unwrap(), vec![Token::NumberF64(4.0), Token::LogicalOp(LogicalOp::And), Token::NumberF64(5.0), Token::LogicalOp(LogicalOp::Or), Token::NumberF64(6.0), Token::EOF]);
}

#[test]
fn test_lex_assignment() {
    let str = "abc = 4";
    assert_eq!(lex(str).unwrap(), vec![Token::Ident("abc".to_string()), Token::Eq, Token::NumberF64(4.0), Token::EOF]);
}
#[test]
fn test_ret() {
    let str = "return 1 return abc";
    assert_eq!(lex(str).unwrap(), vec![Token::Ret, Token::NumberF64(1.0), Token::Ret, Token::Ident(format!("abc")), Token::EOF]);
}

#[test]
fn test_if_else() {
    let str = "if (1) { 2 } else { 3 }";
    assert_eq!(lex(str).unwrap(), vec![Token::If, Token::LeftParen, Token::NumberF64(1.0), Token::RightParen, Token::LeftBrace, Token::NumberF64(2.0), Token::RightBrace, Token::Else, Token::LeftBrace, Token::NumberF64(3.0), Token::RightBrace, Token::EOF]);
}

#[test]
fn test_if_only() {
    let str = "if (1) { 2 }";
    assert_eq!(lex(str).unwrap(), vec![Token::If, Token::LeftParen, Token::NumberF64(1.0), Token::RightParen, Token::LeftBrace, Token::NumberF64(2.0), Token::RightBrace, Token::EOF]);
}
#[test]
fn test_begin_end() {
    let str = "BEGIN begin END end";
    assert_eq!(lex(str).unwrap(), vec![Token::Begin, Token::Begin,Token::End,Token::End,Token::EOF]);
}
#[test]
fn test_ident() {
    let str = "{ x }";
    assert_eq!(lex(str).unwrap(), vec![Token::LeftBrace, Token::Ident("x".to_string()), Token::RightBrace, Token::EOF]);
}

#[test]
fn test_string() {
    let str = "{ \"x\" }";
    assert_eq!(lex(str).unwrap(), vec![Token::LeftBrace, Token::String("x".to_string()), Token::RightBrace, Token::EOF]);
}

#[test]
fn test_string_2() {
    let str = "{ \"abc123 444\" }";
    assert_eq!(lex(str).unwrap(), vec![Token::LeftBrace, Token::String("abc123 444".to_string()), Token::RightBrace, Token::EOF]);
}