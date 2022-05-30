mod types;

pub use types::{Stmt, Expr, Program};
pub use crate::parser::types::{PatternAction};
use crate::lexer::{BinOp, Token, TokenType};


enum PAType {
    Normal(PatternAction),
    Begin(Stmt),
    End(Stmt),
}

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
        let mut begin = vec![];
        let mut end = vec![];
        let mut generic = vec![];
        while !self.is_at_end() {
            match self.pattern_action() {
                PAType::Normal(pa) => generic.push(pa),
                PAType::Begin(pa) => begin.push(pa),
                PAType::End(pa) => end.push(pa),
            }
        }
        Program::new(begin, end, generic)
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
        panic!("{} - didn't find a {} as expected. Found a {} {:?}",
               message,
               TokenType::name(typ),
               TokenType::name(self.peek().ttype()),
               self.peek());
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

    fn pattern_action(&mut self) -> PAType {
        let b = if self.matches(vec![TokenType::LeftBrace]) {
            // { print 1; }
            let pa = PAType::Normal(PatternAction::new_action_only(self.stmts()));
            self.consume(TokenType::RightBrace, "Expected '}' after action block");
            pa
        } else if self.matches(vec![TokenType::Begin]) {
            // BEGIN { print 1; }
            self.consume(TokenType::LeftBrace, "Expected a '{' after a begin");
            let pa = PAType::Begin(self.stmts());
            self.consume(TokenType::RightBrace, "Begin action should end with '}'");
            pa
        } else if self.matches(vec![TokenType::End]) {
            // END { print 1; }
            self.consume(TokenType::LeftBrace, "Expected a {' after a end");
            let pa = PAType::End(self.stmts());
            self.consume(TokenType::RightBrace, "End action should end with '}'");
            pa
        } else {
            let test = self.expression();
            if self.matches(vec![TokenType::LeftBrace]) {
                // test { print 1; }
                let pa = PAType::Normal(PatternAction::new(Some(test), self.stmts()));
                self.consume(TokenType::RightBrace, "Patern action should end with '}'");
                pa
            } else {
                // test
                // ^ implicitly prints line if test passes
                PAType::Normal(PatternAction::new_pattern_only(test))
            }
        };
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
            let stmt = if self.matches(vec![TokenType::Print]) {
                Stmt::Print(self.expression())
            } else if self.peek_next().ttype() == TokenType::Eq {
                let str = if let Token::Ident(str) = self.consume(TokenType::Ident, "Expected identifier before '='") { str } else { panic!("Expected identifier before '='") };
                self.consume(TokenType::Eq, "Expected '=' after identifier");
                Stmt::Assign(str, self.expression())
            } else if self.matches(vec![TokenType::Ret]) {
                self.return_stmt()
            } else if self.matches(vec![TokenType::Print]) {
                let expr = self.expression();
                Stmt::Print(expr)
            } else if self.matches(vec![TokenType::If]) {
                self.if_stmt()
            } else if self.matches(vec![TokenType::LeftBrace]) {
                let s = self.stmts();
                self.consume(TokenType::RightBrace, "Expected a right brace after a group");
                s
            } else {
                Stmt::Expr(self.expression())
            };
            if self.peek().ttype() == TokenType::Semicolon {
                self.consume(TokenType::Semicolon, "not possible");
            }
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
        if self.matches(vec![TokenType::Column]) {
            return Expr::Column(Box::new(self.expression()));
        }
        self.equality()
    }
    // fn str_concat(&mut self) -> Expr {
    //     if self.mat
    // }
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
            Token::NumberF64(num) => {
                self.advance();
                Expr::NumberF64(num)
            }
            Token::NumberI64(num) => {
                self.advance();
                Expr::NumberI64(num)
            }
            Token::LeftParen => {
                self.consume(TokenType::LeftParen, "Expected to parse a left paren here");
                let expr = self.expression();
                self.consume(TokenType::RightParen, "Missing closing ')' after group");
                expr
            }
            Token::Ident(name) => {
                self.consume(TokenType::Ident, "Expected to parse a string here");
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
               Program::new(vec![], vec![], vec![
                   PatternAction::new_action_only(Stmt::Expr(Expr::BinOp(Box::new(Expr::NumberI64(1)), BinOp::Plus, Box::new(Expr::NumberI64(2)))))
               ]));
}


#[test]
fn test_ast_oop() {
    use crate::lexer::lex;
    let left = Box::new(Expr::NumberI64(1));
    let right = Box::new(Expr::BinOp(Box::new(Expr::NumberI64(3)), BinOp::Star, Box::new(Expr::NumberI64(2))));
    let mult = Stmt::Expr(Expr::BinOp(left, BinOp::Plus, right));
    assert_eq!(parse(lex("{1 + 3 * 2;}").unwrap()), Program::new_action_only(mult));
}

#[test]
fn test_ast_oop_2() {
    use crate::lexer::lex;
    let left = Box::new(Expr::NumberI64(2));
    let right = Box::new(Expr::BinOp(Box::new(Expr::NumberI64(1)), BinOp::Star, Box::new(Expr::NumberI64(3))));
    let mult = Stmt::Expr(Expr::BinOp(right, BinOp::Plus, left));
    assert_eq!(parse(lex("{1 * 3 + 2;}").unwrap()), Program::new_action_only(mult));
}


#[test]
fn test_ast_assign() {
    use crate::lexer::lex;
    let stmt = Stmt::Assign(format!("abc"), Expr::NumberF64(2.0));
    assert_eq!(parse(lex("{abc = 2.0; }").unwrap()), Program::new_action_only(stmt));
}

#[test]
fn test_ret() {
    use crate::lexer::lex;
    let stmt = Stmt::Return(Some(Expr::NumberI64(2)));
    assert_eq!(parse(lex("{return 2; }").unwrap()), Program::new_action_only(stmt));
}

#[test]
fn test_ret_nil() {
    use crate::lexer::lex;
    let stmt = Stmt::Return(None);
    assert_eq!(parse(lex("{return;}").unwrap()), Program::new_action_only(stmt));
}

#[test]
fn test_if_else() {
    use crate::lexer::lex;
    let str = "{ if (1) { return 2; } else { return 3; }}";
    let actual = parse(lex(str).unwrap());
    assert_eq!(actual, Program::new_action_only(Stmt::If(Expr::NumberI64(1), Box::new(Stmt::Return(Some(Expr::NumberI64(2)))), Some(Box::new(Stmt::Return(Some(Expr::NumberI64(3))))))));
}

#[test]
fn test_if_only() {
    use crate::lexer::lex;
    let str = "{if (1) { return 2; }}";
    assert_eq!(parse(lex(str).unwrap()), Program::new_action_only(Stmt::If(Expr::NumberI64(1), Box::new(Stmt::Return(Some(Expr::NumberI64(2)))), None)));
}

#[test]
fn test_print() {
    use crate::lexer::lex;
    let str = "{print 1;}";
    assert_eq!(parse(lex(str).unwrap()), Program::new_action_only(Stmt::Print(Expr::NumberI64(1))));
}

#[test]
fn test_group() {
    use crate::lexer::lex;
    let str = "{{print 1; print 2;}}";
    assert_eq!(parse(lex(str).unwrap()), Program::new_action_only(Stmt::Group(vec![Stmt::Print(Expr::NumberI64(1)), Stmt::Print(Expr::NumberI64(2))])));
}


#[test]
fn test_if_else_continues() {
    use crate::lexer::lex;
    let str = "{if (1) { return 2; } else { return 3; } 4.0;}";
    let actual = parse(lex(str).unwrap());
    assert_eq!(actual, Program::new_action_only(
        Stmt::Group(vec![
            Stmt::If(
                Expr::NumberI64(1),
                Box::new(Stmt::Return(Some(Expr::NumberI64(2)))),
                Some(Box::new(Stmt::Return(Some(Expr::NumberI64(3)))))),
            Stmt::Expr(Expr::NumberF64(4.0))])));
}

#[test]
fn test_begin_end() {
    use crate::lexer::lex;
    let str = "a { print 5; } BEGIN { print 1; } begin { print 2; } END { print 3; } end { print 4; }";
    let actual = parse(lex(str).unwrap());
    let begins = vec![Stmt::Print(Expr::NumberI64(1)), Stmt::Print(Expr::NumberI64(2))];
    let ends = vec![Stmt::Print(Expr::NumberI64(3)), Stmt::Print(Expr::NumberI64(4))];
    let generic = PatternAction::new(Some(Expr::Variable("a".to_string())), Stmt::Print(Expr::NumberI64(5)));
    assert_eq!(actual, Program::new(begins, ends, vec![generic]));
}

#[test]
fn test_begin_end2() {
    use crate::lexer::lex;
    let str = "a { print 5; }";
    let actual = parse(lex(str).unwrap());
}

#[test]
fn test_pattern_only() {
    use crate::lexer::lex;
    let str = "test";
    let actual = parse(lex(str).unwrap());
    assert_eq!(actual, Program::new(vec![], vec![], vec![PatternAction::new_pattern_only(Expr::Variable("test".to_string()))]));
}

#[test]
fn test_print_no_semicolon() {
    use crate::lexer::lex;
    let str = "{ print 1 }";
    let actual = parse(lex(str).unwrap());
    assert_eq!(actual, Program::new(vec![], vec![], vec![PatternAction::new_action_only(Stmt::Print(Expr::NumberI64(1)))]));
}

#[test]
fn test_column() {
    use crate::lexer::lex;
    let str = "$0+2 { print a; }";
    let actual = parse(lex(str).unwrap());
    let body = Stmt::Print(Expr::Variable("a".to_string()));
    let two = Box::new(Expr::NumberI64(2));
    let zero = Box::new(Expr::NumberI64(0));
    let pattern = Expr::Column(Box::new(Expr::BinOp(zero, BinOp::Plus, two)));
    let pa = PatternAction::new(Some(pattern), body);
    assert_eq!(actual, Program::new(vec![], vec![], vec![pa]));
}
