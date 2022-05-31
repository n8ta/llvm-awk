use std::io::Write;
use tempfile::{tempdir, tempfile};
use crate::{lex, parse};
use crate::codgen::compile;
use crate::runner::{run, run_and_capture};

const ONE_LINE: &'static str = "1 2 3\n";
const NUMBERS: &'static str = "1 2 3\n4 5 6\n7 8 9";
const FLOAT_NUMBERS: &'static str = "1.1 2.2 3.3\n4.4 5.5 6.6\n7.7 8.8 9.9";

fn run_it(program: &str, file: &str) -> (String, String,  i32) {
    let mut temp_dir = tempdir().unwrap();
    let temp_path = temp_dir.path().join("temp_file");
    std::fs::write(&temp_path, file.as_bytes()).unwrap();

    let temp_path_str = temp_path.to_str().unwrap().to_string();
    run_and_capture(compile(parse(lex(program).unwrap()), &[temp_path_str] , false))
}
fn test_it(program: &str, file: &str, output: &str, status_code: i32) {
    let (stdout, stderr, status) = run_it(program, file);
    if (status != status_code || (status == status_code && stdout != output)) {
        eprintln!("test failed\n{}\n{}", program, file)
    }
    assert_eq!(status, status_code);
    if status_code == 0 {
        assert_eq!(stderr, format!(""));
        assert_eq!(stdout, output);
    }
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
    test_it("{print 1 + 1.1}", ONE_LINE, "", 255);
    test_it("{print 1.1 + 1}", ONE_LINE, "", 255);
    test_it("{print (1.1 + 3.3) + 1}", ONE_LINE, "", 255);
    test_it("{print (1.0 + 2.0)}", ONE_LINE, "3\n", 0);
}


#[test]
fn test_print_columns() {
    test_it("{print $1; print $2; print $3; print $0}", ONE_LINE, "1\n2\n3\n1 2 3\n", 0);
}

