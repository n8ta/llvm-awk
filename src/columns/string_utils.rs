use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Read};
use std::path::PathBuf;
use std::process::id;
use std::str::Utf8Error;

enum ReadError {
    Utf8(Utf8Error),
    Io(std::io::Error),
}

impl Into<ReadError> for Utf8Error {
    fn into(self) -> ReadError {
        ReadError::Utf8(self)
    }
}

impl Into<ReadError> for std::io::Error {
    fn into(self) -> ReadError {
        ReadError::Io(self)
    }
}

pub fn starts_with(haystack: &[u8], needle: &[u8], base_idx: usize) -> bool {
    if (haystack.len() < base_idx || (haystack.len()-base_idx) < needle.len()) { return false; }
    for idx in 0..needle.len() {
        if haystack[base_idx + idx] != needle[idx] {
            return false;
        }
    }
    true
}

#[test]
fn test_starts_with() {
    assert!(starts_with(&[0,1,2,3,4,5], &[0], 0));
    assert!(!starts_with(&[0,1,2,3,4,5], &[0,1,2,3,4,5,6], 0));
    assert!(!starts_with(&[0,1,2,3,4,5], &[0,1,3,3,4,5], 0));
    assert!(starts_with(&[0,1,2,3,4,5], &[0,1,2,3,4,5], 0));
    assert!(starts_with(&[0,1,2,3,4,5], &[1], 1));
    assert!(starts_with(&[0,1,2,3,4,5], &[5], 5));
    assert!(starts_with(&[0,1,2,3,4,5], &[2,3,4,5], 2));
    assert!(!starts_with(&[0,1,2,3,4,5], &[2,3,4,5,6], 2));
    assert!(!starts_with(&[], &[0], 2));
    assert!(!starts_with(&[], &[0], 0));
    assert!(!starts_with(&[], &[], 1));
    assert!(starts_with(&[], &[], 0));
}


fn read_until(existing_buffer: Vec<u8>, mut source: Box<dyn BufRead>, separator: &str) -> Result<String, ReadError> {
    let mut buffer = existing_buffer;
    let mut sep = separator;
    if separator.len() == 0 {
        sep = " ";
    }
    let sep_bytes = sep.as_bytes();
    let mut idx = 0;
    todo!("")
}