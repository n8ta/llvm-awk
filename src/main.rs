use std::env::Args;
use std::process::exit;

use crate::lexer::{BinOp, lex};
use std::io::{Read, Write};
use crate::args::AwkArgs;
use crate::parser::{Expr, parse};
use crate::runner::run;

mod parser;
mod lexer;
mod codgen;
mod test;
mod runner;
mod args;


fn main() {
    let args: Vec<String> = std::env::args().collect();
    let args = match AwkArgs::new(args) {
        Ok(args) => args,
        Err() => return,
    };
    let program = match args.program.load() {
        Ok(program) => program,
        Err(e) => {
            eprintln!("{}", e);
            return;
        }
    };
    let tokens = lex(&program).unwrap();
    let ast = parse(tokens);
    let bitcode = codgen::compile(ast, args.files.as_slice(), args.dump);
    run(bitcode);
}