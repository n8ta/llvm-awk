mod types;

pub use types::{Token, TokenType};

pub fn lex(str: &str) -> Vec<Token> {
    let mut lexer = Lexer::new(str);
    lexer.scan_tokens();
    lexer.tokens
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
        // self.tokens.push(
        //     Token::new_src(
        //         tt,
        //         self.start,
        //         self.current - self.start,
        //         self.line,
        //         self.source.clone(),
        //     )
        // );
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
        // let sym = self.symbolizer.get_symbol(str);
        self.add_token(Token::String(str));
        return Ok(());
    }
    fn number(&mut self) -> Result<(), String> {
        while self.peek().is_digit(10) { self.advance(); }
        if self.peek() == '.' && self.peek_next().is_digit(10) {
            self.advance();
        }
        while self.peek().is_digit(10) { self.advance(); }

        let num = self.src.chars().skip(self.start).take(self.current - self.start).collect::<String>();
        let float = match num.parse::<f64>() {
            Ok(float) => float,
            Err(_) => {
                return Err(String::from("Unable to parse f64 {}"));
            }
        };
        self.add_token(Token::Number(float));
        Ok(())
    }
    fn identifier(&mut self) -> Result<(), String> {
        while self.peek().is_alphanumeric() { self.advance(); }
        // let ident = self.src.chars().skip(self.start).take(self.current - self.start).collect::<String>();
        // let fetched: Option<Token> = self.keywords.get(&ident).and_then(|t| Some(t.clone()));
        // match fetched {
        //     Some(k) => self.add_token(k.clone()),
        //     None => {
        let src = self.src.chars().skip(self.start).take(self.current - self.start).collect();
        if src == "true" {
            self.add_token(Token::True);
        } else if src == "false" {
            self.add_token(Token::False);
        } else {
            self.add_token(Token::String(src));
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
            // '(' => self.add_token(Token::LeftParen),
            // ')' => self.add_token(Token::RightParen),
            // '{' => self.add_token(Token::LeftBrace),
            // '}' => self.add_token(Token::RightBrace),
            // ',' => self.add_token(Token::Comma),
            // '.' => self.add_token(Token::Dot),
            '-' => self.add_token(Token::Minus),
            '+' => self.add_token(Token::Plus),
            // ';' => self.add_token(Token::Semicolon),
            '*' => self.add_token(Token::Star),
            '!' => {
                let tt = match self.matches('=') {
                    true => Token::BangEq,
                    false => Token::Bang,
                };
                self.add_token(tt);
            }
            '|' => {
                let tt = match self.matches('|') {
                    true => Token::Or,
                    false => return Err("| must be followed by ||".to_string()),
                };
                self.add_token(tt);
            }
            '&' => {
                let tt = match self.matches('&') {
                    true => Token::And,
                    false => return Err("| must be followed by &".to_string()),
                };
                self.add_token(tt);
            }
            '=' => {
                let tt = match self.matches('=') {
                    true => Token::EqEq,
                    false => todo!("assignment") //Token::Eq
                };
                self.add_token(tt)
            }
            '<' => {
                let tt = match self.matches('=') {
                    true => Token::LessEq,
                    false => Token::Less
                };
                self.add_token(tt)
            }
            '>' => {
                let tt = match self.matches('=') {
                    true => Token::GreaterEq,
                    false => Token::Greater
                };
                self.add_token(tt)
            }
            '/' => {
                if self.matches('/') {
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else {
                    self.add_token(Token::Slash);
                }
            }
            '"' => self.string()?,
            '\r' => (),
            '\t' => (),
            ' ' => (),
            '\n' => self.line += 1,
            _ => {
                if c.is_digit(10) {
                    self.number()?;
                } else if c.is_alphabetic() {
                    self.identifier()?;
                } else {
                    return Err(format!("Unexpected token `{}`", c));
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
fn test_lex_binops_and_true_false() {
    let str = "4*2+1-2+false/true";
    let tokens = lex(str);
    assert_eq!(tokens, vec![Token::Number(4.0), Token::Star, Token::Number(2.0), Token::Plus, Token::Number(1.0), Token::Minus, Token::Number(2.0), Token::Plus, Token::False, Token::Slash, Token::True, Token::EOF]);
}

#[test]
fn test_lex_decimals() {
    let str = "4.123-123.123";
    assert_eq!(lex(str), vec![Token::Number(4.123), Token::Minus, Token::Number(123.123), Token::EOF]);
}

#[test]
fn test_lex_equality() {
    let str = "4 != 5 == 6";
    assert_eq!(lex(str), vec![Token::Number(4.0), Token::BangEq, Token::Number(5.0), Token::EqEq, Token::Number(6.0), Token::EOF]);
}

#[test]
fn test_lex_logical_op() {
    let str = "4 && 5 || 6";
    assert_eq!(lex(str), vec![Token::Number(4.0), Token::And, Token::Number(5.0), Token::Or, Token::Number(6.0), Token::EOF]);
}