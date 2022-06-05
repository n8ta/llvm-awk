use tempfile::{tempdir};
use crate::{lex, parse, transform};
use crate::codgen::compile;
use crate::runner::{run_and_capture};

const ONE_LINE: &'static str = "1 2 3\n";
const NUMBERS: &'static str = "1 2 3\n4 5 6\n7 8 9";
const FLOAT_NUMBERS: &'static str = "1.1 2.2 3.3\n4.4 5.5 6.6\n7.7 8.8 9.9";

fn run_it(program: &str, file: &str) -> (String, String, i32) {
    let temp_dir = tempdir().unwrap();
    let temp_path = temp_dir.path().join("temp_file");
    std::fs::write(&temp_path, file.as_bytes()).unwrap();
    let temp_path_str = temp_path.to_str().unwrap().to_string();
    let r = run_and_capture(compile(transform(parse(lex(program).unwrap())), &[temp_path_str], false));
    let contents = std::fs::read_to_string(temp_path).unwrap();
    println!(" temp file contents: {:?}", contents);
    r
}

fn test_it(program: &str, file: &str, output: &str, status_code: i32) {
    println!("====PROGRAM====\n{}\n=====DATA=====\n{}\n=====EXPECTED======\n{}============", program, file, output);

    let (stdout, stderr, status) = run_it(program, file);
    println!("=====STATUS {}======\n=======STDOUT======\n{}======STDERRR======\n{}=====EXPECTED======\n{}", status, stdout.replace("\\n", "\n"), stderr.replace("\\n", "\n"), output);
    println!("test complete for {}", program);
    assert_eq!(status, status_code);
    if status_code == 0 {
        assert_eq!(stdout, output);
        assert_eq!(0, stderr.len());
    }
}

#[test]
fn test_e2e() {
    test_it("{print 1;}", ONE_LINE, "1\n", 0);
}


#[test]
fn test_1() {
    test_it("BEGIN { print 1; } END { print 3; } END { print 4; }", ONE_LINE, "1\n3\n4\n", 0);
}

#[test]
fn test_2() {
    test_it("END { print 3; } { print 2; } BEGIN {print 1;}", ONE_LINE, "1\n2\n3\n", 0);
}

#[test]
fn test_3() {
    test_it("end { print 4; } END { print 3; } { print 2; } begin { print 0; } BEGIN {print 1;} ", ONE_LINE, "0\n1\n2\n4\n3\n", 0);
}

#[test]
fn test_4() {
    test_it("{print 1;}", ONE_LINE, "1\n", 0);
}

#[test]
fn test_5() {
    test_it("{x = 0; print x;}", ONE_LINE, "0\n", 0);
}

#[test]
fn test_6() {
    test_it("{x = 0; if (1) { x = 1 } else { x = 2.2 }; print x }", ONE_LINE, "1\n", 0);
}

#[test]
fn test_7() {
    test_it("{x = 0; if (0) { x = 1 } else { x = 2.2 }; print x }", ONE_LINE, "2.2\n", 0);
}

#[test]
fn test_8() {
    test_it("{x = 0; if (x) { x = 1 } else { x = 2.2 }; print x }", ONE_LINE, "2.2\n", 0);
}

#[test]
fn test_9() {
    test_it("{x = 0; if (1) { if (1) { x = 1 } else { x = 2.2 } } else { if (1) { x = 1 } else { x = 4.2 } }; print x }", ONE_LINE, "1\n", 0);
}

#[test]
fn test_10() {
    test_it("{print 1 + 1.1}", ONE_LINE, "2.1\n", 0);
}

#[test]
fn test_11() {
    test_it("{print 1.1 + 1}", ONE_LINE, "2.1\n", 0);
}

#[test]
fn test_12() {
    test_it("{print (1.1 + 3.3) + 1}", ONE_LINE, "5.4\n", 0);
}

#[test]
fn test_13() {
    test_it("{print (1.0 + 2.0)}", ONE_LINE, "3\n", 0);
}

#[test]
fn test_print_columns() {
    test_it("{print $1; print $2; print $3; print $0}", ONE_LINE, "1\n2\n3\n1 2 3\n", 0);
}

