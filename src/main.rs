use crate::args::AwkArgs;
use crate::lexer::{BinOp, lex};
use crate::parser::{Expr, parse};
use crate::runner::run;
use crate::transformer::transform;

mod parser;
mod lexer;
mod codgen;
#[allow(dead_code)]
mod test;
mod runner;
mod args;
mod transformer;


fn main() {
    let args: Vec<String> = std::env::args().collect();
    let args = match AwkArgs::new(args) {
        Ok(args) => args,
        Err(_)=> return,
    };
    let program = match args.program.load() {
        Ok(program) => program,
        Err(e) => {
            eprintln!("{}", e);
            return;
        }
    };
    let ast = transform(parse(lex(&program).unwrap()));
    let bitcode = codgen::compile(ast, args.files.as_slice(), args.dump);
    run(bitcode, args.save_executable);
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