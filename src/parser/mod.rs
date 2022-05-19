mod types;

pub use types::{Stmt, Expr, Program};
use crate::lexer::{BinOp, Token, TokenType};
use crate::parser::types::Block;


pub fn parse(tokens: Vec<Token>) -> Program {
    let mut parser = Parser { tokens, current: 0 };
    parser.parse()
}

struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser { tokens, current: 0 }
    }

    fn parse(&mut self) -> Program {
        let mut blocks: Vec<Block> = vec![];
        while !self.is_at_end() {
            println!("block!");
            blocks.push(self.block())
        }
        Program::new(blocks)
    }

    fn check(&mut self, typ: TokenType) -> bool {
        if self.is_at_end() {
            false
        } else {
            typ == self.peek().ttype()
        }
    }

    fn consume(&mut self, typ: TokenType, message: &str) -> Token {
        if self.check(typ.clone()) { return self.advance(); }
        panic!("{} - didn't find a {} as expected. Found a {}",
               message,
               TokenType::name(typ),
               TokenType::name(self.peek().ttype()));
    }

    fn matches(&mut self, tokens: Vec<TokenType>) -> bool {
        let tkn = match self.tokens.get(self.current) {
            None => return false,
            Some(t) => t.ttype().clone(),
        };
        for expected in tokens.iter() {
            if *expected == tkn {
                self.advance();
                return true;
            }
        }
        false
    }

    fn previous(&self) -> Option<Token> {
        if self.current == 0 {
            return None;
        }
        Some(self.tokens[self.current - 1].clone())
    }

    fn peek(&self) -> Token {
        return self.tokens[self.current].clone();
    }

    fn is_at_end(&self) -> bool {
        self.tokens[self.current].ttype() == TokenType::EOF
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            println!("advancing...");
            self.current += 1;
        }
        self.previous().unwrap()
    }

    fn block(&mut self) -> Block {
        let b = if self.matches(vec![TokenType::LeftBrace]) {
            Block::new(None, self.stmts())
        } else {
            Block::new(Some(self.expression()), self.stmts())
        };
        self.consume(TokenType::RightBrace, "Block ends with }");
        b
    }
    fn stmts(&mut self) -> Vec<Stmt> {
        let mut first = true;
        let mut stmts = vec![];
        while self.peek().ttype() != TokenType::RightBrace {
            if !first {
                self.consume(TokenType::Semicolon, &format!("Expected a ';' after a statement. Found a {}", TokenType::name(self.peek().ttype())));
            }
            first = false;
            let stmt = if self.matches(vec![TokenType::Print]) {
                Stmt::Print(self.expression())
            } else {
                Stmt::Expr(self.expression())
            };
            stmts.push(stmt);
        }
        stmts
    }
    fn expression(&mut self) -> Expr {
        self.equality()
    }
    fn equality(&mut self) -> Expr {
        let mut expr = self.comparison();
        while self.matches(vec![TokenType::EqEq, TokenType::BangEq]) {
            let op = match self.previous().unwrap() {
                Token::BinOp(BinOp::EqEq) => BinOp::EqEq,
                Token::BinOp(BinOp::BangEq) => BinOp::BangEq,
                _ => panic!("Parser bug in equality function")
            };
            expr = Expr::BinOp(Box::new(expr), op, Box::new(self.comparison()))
        }
        expr
    }
    fn comparison(&mut self) -> Expr {
        let mut expr = self.term();
        println!("term got {:?}", expr);
        while self.matches(vec![TokenType::Plus, TokenType::Minus]) {
            let op = match self.previous().unwrap() {
                Token::BinOp(BinOp::Minus) => BinOp::Minus,
                Token::BinOp(BinOp::Plus) => BinOp::Plus,
                _ => panic!("Parser bug in comparison function")
            };
            expr = Expr::BinOp(Box::new(expr), op, Box::new(self.term()))
        }
        expr
    }

    fn term(&mut self) -> Expr {
        let mut expr = self.primary();
        while self.matches(vec![TokenType::Star, TokenType::Slash]) {
            let op = match self.previous().unwrap() {
                Token::BinOp(BinOp::Star) => BinOp::Star,
                Token::BinOp(BinOp::Slash) => BinOp::Slash,
                _ => panic!("Parser bug in comparison function")
            };
            expr = Expr::BinOp(Box::new(expr), op, Box::new(self.primary()))
        }
        expr
    }

    fn primary(&mut self) -> Expr {
        if self.is_at_end() {
            panic!("Primary and at end")
        }
        match self.tokens.get(self.current).unwrap().clone() {
            Token::Number(num) => {
                self.advance();
                Expr::Number(num)
            },
            Token::LeftParen => {
                self.consume(TokenType::LeftParen, "Expected to parse a left paren here");
                let expr = self.expression();
                self.consume(TokenType::RightParen, "Missing closing ')' after group");
                expr
            }
            t => panic!("Unexpected token {}", TokenType::name(t.ttype()))
        }
    }
}

#[test]
fn test_ast_number() {
    use crate::lexer::lex;

    assert_eq!(parse(lex("{1 + 2}").unwrap()),
               Program::new(vec![
                   Block::new(None, vec![Stmt::Expr(Expr::BinOp(Box::new(Expr::Number(1.0)), BinOp::Plus, Box::new(Expr::Number(2.0))))])
               ]));
}


























