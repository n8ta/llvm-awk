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

    fn peek_next(&self) -> Token {
        return self.tokens[self.current + 1].clone();
    }

    fn is_at_end(&self) -> bool {
        self.tokens[self.current].ttype() == TokenType::EOF
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
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
    fn group(&mut self) -> Stmt {
        self.consume(TokenType::LeftBrace, "Expected a '}'");
        let s = self.stmts();
        self.consume(TokenType::RightBrace, "Expected a '}'");
        s
    }

    fn stmts(&mut self) -> Stmt {
        let mut stmts = vec![];
        while self.peek().ttype() != TokenType::RightBrace {
            println!("{:?}", self.peek());
            let stmt = if self.matches(vec![TokenType::Print]) {
                let s = Stmt::Print(self.expression());
                self.consume(TokenType::Semicolon, "Expected ; after print");
                s
            } else if self.peek_next().ttype() == TokenType::Eq {
                let str = if let Token::String(str) = self.consume(TokenType::String, "Expected identifier before '='") { str } else { panic!("Expected identifier before '='") };
                self.consume(TokenType::Eq, "Expected '=' after identifier");
                let s = Stmt::Assign(str, self.expression());
                self.consume(TokenType::Semicolon, "Expected ';' after '='");
                s
            } else if self.matches(vec![TokenType::Ret]) {
                let s = self.return_stmt();
                self.consume(TokenType::Semicolon, "Expected ';' after return statement");
                s
            } else if self.matches(vec![TokenType::If]) {
                self.if_stmt()
            } else {
                let s = Stmt::Expr(self.expression());
                self.consume(TokenType::Semicolon, "Expected ';' after statement");
                s
            };
            stmts.push(stmt);
        }
        if stmts.len() == 1 {
            return stmts.pop().unwrap();
        }
        Stmt::Group(stmts)
    }
    fn return_stmt(&mut self) -> Stmt {
        if self.peek().ttype() == TokenType::Semicolon {
            Stmt::Return(None)
        } else {
            Stmt::Return(Some(self.expression()))
        }
    }
    fn if_stmt(&mut self) -> Stmt {
        println!("if statement");
        self.consume(TokenType::LeftParen, "Expected '(' after if");
        let predicate = self.expression();
        self.consume(TokenType::RightParen, "Expected ')' after if predicate");
        let then_blk = self.group();
        let else_blk = if self.matches(vec![TokenType::Else]) {
            Some(Box::new(self.group()))
        } else {
            None
        };
        Stmt::If(predicate, Box::new(then_blk), else_blk)
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
            }
            Token::LeftParen => {
                self.consume(TokenType::LeftParen, "Expected to parse a left paren here");
                let expr = self.expression();
                self.consume(TokenType::RightParen, "Missing closing ')' after group");
                expr
            }
            Token::String(name) => {
                self.consume(TokenType::String, "Expected to parse a string here");
                Expr::Variable(name)
            }
            t => panic!("Unexpected token {:?} {}", t, TokenType::name(t.ttype()))
        }
    }
}

#[test]
fn test_ast_number() {
    use crate::lexer::lex;

    assert_eq!(parse(lex("{1 + 2;}").unwrap()),
               Program::new(vec![
                   Block::new(None,
                              Stmt::Expr(Expr::BinOp(Box::new(Expr::Number(1.0)), BinOp::Plus, Box::new(Expr::Number(2.0)))))
               ]));
}


#[test]
fn test_ast_oop() {
    use crate::lexer::lex;
    let left = Box::new(Expr::Number(1.0));
    let right = Box::new(Expr::BinOp(Box::new(Expr::Number(3.0)), BinOp::Star, Box::new(Expr::Number(2.0))));
    let mult = Stmt::Expr(Expr::BinOp(left, BinOp::Plus, right));
    assert_eq!(parse(lex("{1 + 3 * 2;}").unwrap()), Program::new(vec![Block::new(None, mult)]));
}

#[test]
fn test_ast_oop_2() {
    use crate::lexer::lex;
    let left = Box::new(Expr::Number(2.0));
    let right = Box::new(Expr::BinOp(Box::new(Expr::Number(1.0)), BinOp::Star, Box::new(Expr::Number(3.0))));
    let mult = Stmt::Expr(Expr::BinOp(right, BinOp::Plus, left));
    assert_eq!(parse(lex("{1 * 3 + 2;}").unwrap()), Program::new(vec![Block::new(None, mult)]));
}


#[test]
fn test_ast_assign() {
    use crate::lexer::lex;
    let stmt = Stmt::Assign(format!("abc"), Expr::Number(2.0));
    assert_eq!(parse(lex("{abc = 2.0; }").unwrap()), Program::new(vec![Block::new(None, stmt)]));
}

#[test]
fn test_ret() {
    use crate::lexer::lex;
    let stmt = Stmt::Return(Some(Expr::Number(2.0)));
    assert_eq!(parse(lex("{return 2; }").unwrap()), Program::new(vec![Block::new(None, stmt)]));
}

#[test]
fn test_ret_nil() {
    use crate::lexer::lex;
    let stmt = Stmt::Return(None);
    assert_eq!(parse(lex("{return;}").unwrap()), Program::new(vec![Block::new(None, stmt)]));
}

#[test]
fn test_if_else() {
    use crate::lexer::lex;
    let str = "{ if (1) { return 2; } else { return 3; }}";
    let actual = parse(lex(str).unwrap());
    assert_eq!(actual, Program::new(vec![Block::new(None, Stmt::If(Expr::Number(1.0), Box::new(Stmt::Return(Some(Expr::Number(2.0)))), Some(Box::new(Stmt::Return(Some(Expr::Number(3.0)))))))]));
}

#[test]
fn test_if_only() {
    use crate::lexer::lex;
    let str = "{if (1) { return 2; }}";
    assert_eq!(parse(lex(str).unwrap()), Program::new(vec![Block::new(None, Stmt::If(Expr::Number(1.0), Box::new(Stmt::Return(Some(Expr::Number(2.0)))), None))]));
}


#[test]
fn test_if_else_continues() {
    use crate::lexer::lex;
    let str = "{if (1) { return 2; } else { return 3; } 4.0;}";
    let actual = parse(lex(str).unwrap());
    assert_eq!(actual, Program::new(vec![Block::new(None,
                                                    Stmt::Group(vec![
                                                        Stmt::If(
                                                            Expr::Number(1.0),
                                                            Box::new(Stmt::Return(Some(Expr::Number(2.0)))),
                                                            Some(Box::new(Stmt::Return(Some(Expr::Number(3.0)))))),
                                                        Stmt::Expr(Expr::Number(4.0))]))]));
}
