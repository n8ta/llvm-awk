mod string_utils;

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Read};
use std::path::PathBuf;
use std::process::id;
use std::str::Utf8Error;

type Line = HashMap<usize, String>;

pub struct Columns {
    rs: String,
    fs: String,
    files: Vec<String>,
    current_path: Option<String>,
    lines: HashMap<usize, Line>,
    line_number: usize,
}

impl Columns {
    pub fn new(files: Vec<String>) -> Self {
        let mut c = Columns {
            rs: String::from("\n"),
            fs: String::from(" "),
            files: files.into_iter().rev().collect(),
            line_number: 0,
            lines: HashMap::new(),
            current_path: None,
        };
        c.next_line();
        c
    }
    pub fn get(&mut self, column: usize) -> String {
        if let Some(line) = self.lines.get(&self.line_number) {
            if let Some(field) = line.get(&column) {
                return field.to_string();
            }
        }
        "".to_string()
    }

    pub fn set(&mut self, column: usize, data: String) {
        if let Some(line) = self.lines.get_mut(&self.line_number) {
            line.insert(column, data);
        } else {
            let mut map = HashMap::new();
            map.insert(column, data);
            self.lines.insert(0, map);
        }
    }

    fn string_to_vec_vec(&self, contents: String) -> HashMap<usize, Line> {
        let mut lines = HashMap::new();
        for (line_idx, line) in contents
            .split(&self.rs)
            .enumerate() {
            let mut map = HashMap::new();
            map.insert(0, line.to_string());
            for (field_idx, field) in line.split(&self.fs).enumerate() {
                map.insert(field_idx + 1, field.to_string());
            }
            lines.insert(line_idx, map);
        }
        println!("{:?}", lines);
        lines
    }


    fn advance_file(&mut self) -> bool {
        if let Some(next_file) = self.files.pop() {
            println!("starting file {}", next_file);
            let contents = std::fs::read_to_string(PathBuf::from(next_file.clone())).unwrap();
            println!("contents: {}",contents);
            self.current_path = Some(next_file);
            self.lines = self.string_to_vec_vec(contents);
            true
        } else {
            false
        }
    }

    pub fn next_line(&mut self) -> bool {
        if self.current_path.is_none() && !self.advance_file() {
            return false;
        }
        loop {
            if let Some(_next_line) = self.lines.get(&self.line_number) {
                self.line_number += 1;
                return true;
            }
            if self.advance_file() {
                self.line_number = 0;
            } else {
                return false;
            }
        }
    }

    fn calc_columns(&mut self, path: &String) {
        let contents = std::fs::read_to_string(PathBuf::from(path)).unwrap();
        self.lines = self.string_to_vec_vec(contents);
    }
    pub fn set_record_sep(&mut self, value: String) {
        if self.current_path.is_some() { panic!("must set fs/rs before reading lines") }
        self.rs = value;
    }
    pub fn set_field_sep(&mut self, value: String) {
        if self.current_path.is_some() { panic!("must set fs/rs before reading lines") }
        self.fs = value;
    }
}

#[test]
fn test_files() {
    use tempfile::{tempdir};

    let temp_dir = tempdir().unwrap();
    let file_path_1 = temp_dir.path().join("file1.txt");
    let file_path_2 = temp_dir.path().join("file2.txt");
    std::fs::write(file_path_1.clone(), "a b c\nd e f\ng h i\n").unwrap();
    std::fs::write(file_path_2.clone(), "1 2 3\n4 5 6\n7 8 9\n").unwrap();

    let mut cols = Columns::new(vec![
        file_path_1.to_str().unwrap().to_string(),
        file_path_2.to_str().unwrap().to_string()]);
    assert!(cols.next_line());
    assert_eq!(cols.get(0), "a b c\n");
    assert_eq!(cols.get(1), "a");
    assert_eq!(cols.get(2), "b");
    assert_eq!(cols.get(3), "c");
    assert!(cols.next_line());
    assert_eq!(cols.get(3), "f");
    assert_eq!(cols.get(2), "e");
    assert_eq!(cols.get(1), "d");
    assert_eq!(cols.get(0), "d e f\n");
}