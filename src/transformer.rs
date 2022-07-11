use crate::{Expr, parser};
use crate::parser::{Stmt, TypedExpr};

pub fn transform(program: parser::Program) -> Stmt {
    let mut prog = program.begins;

    let mut every_line_stms = vec![];
    for pattern in program.pattern_actions {
        let stmt = if let Some(test) = pattern.pattern {
            Stmt::While(test, Box::new(pattern.action))
        } else {
            pattern.action
        };
        every_line_stms.push(stmt)
    }
    if every_line_stms.len() > 0 {
        let line_loop = Stmt::While(TypedExpr::new_num(Expr::Call), Box::new(Stmt::Group(every_line_stms)));
        prog.push(line_loop);
    }

    for end in program.ends {
        prog.push(end);
    }
    Stmt::Group(prog)
}