use std::process::exit;

use crate::lexer::{BinOp, lex};
use std::io::Read;
use crate::parser::{Expr, parse};

mod parser;
mod lexer;
mod codgen;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let path = if let Some(path) = args.get(1) {
        path
    } else {
        println!("Usage: ./crawk something.awk input.txt");
        exit(-1);
    };

    let mut file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(err) => {
            eprintln!("Unable to open file @ '{}'\n Error: {}", path, err);
            exit(-1)
        }
    };

    let mut contents: String = String::new();
    if let Err(err) = file.read_to_string(&mut contents) {
        eprintln!("Unable to read file @ '{}'\nError: {}", path, err);
        exit(-1);
    }

    let tokens = lex(&contents).unwrap();
    let program = parse(tokens);

    codgen::compile_and_run(program);
}