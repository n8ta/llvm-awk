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
    // let args = AwkArgs {
    //     dump: false,
    //     program: ProgramType::CLI("{print $1}".to_string()),
    //     files: vec!["data.txt".to_string()],
    //     save_executable: None,
    // };
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

// use crate::lexer::{BinOp, lex};
// use crate::args::AwkArgs;
// use crate::parser::{Expr, parse};
// use crate::runner::run;
// use crate::transformer::transform;
//
// mod parser;
// mod lexer;
// mod codgen;
// mod test;
// mod runner;
// mod args;
// mod transformer;
//

// fn main() {
//     // let args: Vec<String> = std::env::args().collect();
//     // let args = match AwkArgs::new(args) {
//     //     Ok(args) => args,
//     //     Err(err) => return,
//     // };
//     // let program = match args.program.load() {
//     //     Ok(program) => program,
//     //     Err(e) => {
//     //         eprintln!("{}", e);
//     //         return;
//     //     }
//     // };
//     let program = "{print $1}";
//     let tokens = lex(&program).unwrap();
//     let ast = transform(parse(tokens));
//     let bitcode = codgen::compile(ast, &[format!("/Users/n8ta/code/crawk/data.txt")], true);
//     run(bitcode);
// }