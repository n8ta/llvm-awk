use crate::{lex, parse};
use crate::codgen::compile;
use crate::runner::run;

const ONE_LINE: &'static str = "1 2 3\n";
const NUMBERS: &'static str = "1 2 3\n4 5 6\n7 8 9";
const FLOAT_NUMBERS: &'static str = "1.1 2.2 3.3\n4.4 5.5 6.6\n7.7 8.8 9.9";

fn run_it(program: &str, file: &str) -> (String, String,  i32) {
    run(compile(parse(lex(program).unwrap()), false))
}
fn test_it(program: &str, file: &str, output: &str, status_code: i32) {
    let (stdout, stderr, status)  = run_it(program, file);
    assert_eq!(stderr, format!(""));
    assert_eq!(status, status_code);
    assert_eq!(stdout, output);
}

#[test]
fn test_e2e() {
    test_it("{print 1;}", ONE_LINE, "1\n", 0);
    test_it("END { print 3; } { print 2; } BEGIN {print 1;}", ONE_LINE, "1\n2\n3\n", 0);
    test_it("end { print 4; } END { print 3; } { print 2; } begin { print 0; } BEGIN {print 1;} ", ONE_LINE, "0\n1\n2\n4\n3\n", 0);
    test_it("{print 1;}", ONE_LINE, "1\n", 0);
    test_it("{x = 0; print x;}", ONE_LINE, "0\n", 0);
    test_it("{x = 0; if (1) { x = 1 } else { x = 2.2 }; print x }", ONE_LINE, "2.2\n", 0);
    test_it("{x = 0; if (1) { if (1) { x = 1 } else { x = 2.2 } } else { if (1) { x = 1 } else { x = 4.2 } }; print x }", ONE_LINE, "4.2\n", 0);
}


