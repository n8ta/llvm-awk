use std::process::exit;

use crate::lexer::{BinOp, lex};
use std::io::{Read, Write};
use crate::parser::{Expr, parse};
use crate::runner::run;

mod parser;
mod lexer;
mod codgen;
mod test;
mod runner;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let dump = args.contains(&format!("--dump"));
    let path = if let Some(path) = args.get(1) {
        path
    } else {
        println!("Usage: ./crawk something.awk input1.txt input2.txt`");
        exit(-1);
    };

    let mut file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(err) => {
            eprintln!("Unable to open file @ '{}'\n Error: {}", path, err);
            exit(-1)
        }
    };

    let files: Vec<String> = args[2..].iter().filter(|arg| **arg != "--dump").cloned().collect();
    println!("files {:?}", files);

    let mut contents: String = String::new();
    file.read_to_string(&mut contents).expect("couldnt read source file");
    let tokens = lex(&contents).unwrap();
    let program = parse(tokens);

    let bitcode = codgen::compile(program, files.as_slice(), dump);
    run(bitcode);
    // std::process::exit(status);
    // let (stdout, stderr, exit) = runner::run(bitcode);
    // println!("{}", stdout);
    // eprintln!("{}", stderr);
    // std::process::exit(exit)
}