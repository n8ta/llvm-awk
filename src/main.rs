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
mod columns;


fn main() {
    let args: Vec<String> = std::env::args().collect();
    let args = match AwkArgs::new(args) {
        Ok(args) => args,
        Err(_) => return,
    };
    let args = AwkArgs {
        debug: true,
        program: ProgramType::CLI("{ a = $1; }".to_string()),
        files: vec!["numbers.txt".to_string()],
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

    if args.debug {
        codgen::compile_and_capture(ast, &args.files);
    } else {
        codgen::compile_and_run(ast, &args.files);
    }

}
