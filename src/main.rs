extern crate core;

use crate::args::{AwkArgs, ProgramType};
use crate::lexer::{lex};
use crate::parser::{Expr, parse};
use crate::transformer::transform;
use crate::typing::analyze;

mod parser;
mod lexer;
mod codgen;
#[allow(dead_code)]
mod test;
mod args;
mod transformer;
mod runtime;
mod typing;


fn main() {
    let args: Vec<String> = std::env::args().collect();
    let args = match AwkArgs::new(args) {
        Ok(args) => args,
        Err(_) => return,
    };
    let args = AwkArgs {
        dump: true,
        program: ProgramType::CLI("BEGIN {x = 4.4; if (1) { x=4.4;} else { x = 3.2 }; print x }".to_string()),
        files: vec![],
        save_executable: None,
    };
    let program = match args.program.load() {
        Ok(program) => program,
        Err(e) => {
            eprintln!("{}", e);
            return;
        }
    };
    let mut ast = transform(
        parse(
            lex(&program).unwrap()));

    analyze(&mut ast);
    codgen::compile_and_run(ast, &args.files, args.dump);
}
