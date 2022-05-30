use std::arch::asm;
use std::path::{Path, PathBuf};
use inkwell::memory_buffer::MemoryBuffer;
use std::fs::File;
use std::io::{self, Write};
use tempdir::TempDir;

pub fn run(bitcode: MemoryBuffer) -> (String, String, i32) {
    let temp_dir = TempDir::new("llvm-awk").unwrap();


    let mut bitcode_path = temp_dir.path().join("awk.bc");

    let mut asm_path = temp_dir.path().join("awk.asm");

    let mut out_path = temp_dir.path().join("a.out");

    let dylib = "/Users/n8ta/code/crawk/runtime/target/release/libruntime.dylib";

    {
        let mut file = File::create(bitcode_path.clone()).unwrap();
        file.write_all(bitcode.as_slice()).expect(&format!("could not write to {}", bitcode_path.to_str().unwrap()));
    }

    let res = std::process::Command::new("/Users/n8ta/llvm-13/bin/llc")
        .args(vec![bitcode_path.to_str().unwrap(), "-o", asm_path.to_str().unwrap()])
        .output().unwrap();

    if res.status.code() != Some(0) {
        eprintln!("llc failed to compile into assembly");
        return (String::from_utf8(res.stdout).unwrap(), String::from_utf8(res.stderr).unwrap(), res.status.code().unwrap());
    }

    let res = std::process::Command::new("clang")
        .args(vec![asm_path.to_str().unwrap(), dylib, "-o", out_path.to_str().unwrap()])
        .output().unwrap();

    if res.status.code() != Some(0) {
        eprintln!("clang failed to link with runtime library");
        return (String::from_utf8(res.stdout).unwrap(), String::from_utf8(res.stderr).unwrap(), res.status.code().unwrap());
    }

    let res = std::process::Command::new(out_path).output().unwrap();
    return (String::from_utf8(res.stdout).unwrap(), String::from_utf8(res.stderr).unwrap(), res.status.code().unwrap());
}