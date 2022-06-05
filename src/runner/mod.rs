use std::path::{PathBuf};
use inkwell::memory_buffer::MemoryBuffer;
use std::fs::File;
use std::io::{Read, Write};
use tempfile::{tempdir, TempDir};

const RUNTIME_BITCODE: &[u8] = std::include_bytes!("../../runtime.bc");

pub fn run(bitcode: MemoryBuffer) {
    let temp_dir = tempdir().unwrap();
    match external_tools(&temp_dir, bitcode) {
        Ok(out_path) => {
            let mut child = std::process::Command::new(out_path)
                .spawn()
                .expect("to launch awk process");
            child.wait().expect("for awk program to complete normally");
        }
        Err(err) => {
            println!("{}", err.0);
            eprintln!("{}", err.1);
        }
    }
}

pub fn run_and_capture(bitcode: MemoryBuffer) -> (String, String, i32) {
    let temp_dir = TempDir::new().unwrap();
    match external_tools(&temp_dir, bitcode) {
        Ok(out_path) => {
            println!("out_path {:?}", out_path);
            let res = std::process::Command::new(out_path).output().unwrap();
            println!("res: {:?}", res);
            return (String::from_utf8(res.stdout).expect("stdout"), String::from_utf8(res.stderr).expect("stderr"), res.status.code().or(Some(255)).unwrap());
        }
        Err(err) => return err,
    }
}

pub fn external_tools(temp_dir: &TempDir, bitcode: MemoryBuffer) -> Result<PathBuf, (String, String, i32)> {
    let program_bc_path = temp_dir.path().join("awk.bc");
    let runtime_bc_path = temp_dir.path().join("runtime.bc");
    let out_path = temp_dir.path().join("a.out");


    {
        let mut file = File::create(program_bc_path.clone()).unwrap();
        file.write_all(bitcode.as_slice()).expect(&format!("could not write to {}", program_bc_path.to_str().unwrap()));
    }
    {
        let mut file = File::create(runtime_bc_path.clone()).unwrap();
        file.write_all(RUNTIME_BITCODE).expect(&format!("could not write to {}", runtime_bc_path.to_str().unwrap()));
    }

    let args = vec![runtime_bc_path.to_str().unwrap(), program_bc_path.to_str().unwrap(), "-o", out_path.to_str().unwrap()];
    println!("clang++ {:?}", args);
    let res = std::process::Command::new("clang++")
        .args(args)
        .output().expect("to be able to link with clang");

    if res.status.code() != Some(0) {
        eprintln!("clang++ failed to compile bitcode");
        if let Ok(val) = std::env::var("LLVM_SYS_130_PREFIX") {
            // llc gives helpful error messages for bitcode unlike clang. So if clang dies make
            // an attempt to call LLC llc and print its explanation of the failure
            let mut path = PathBuf::from(val);
            path.push("bin");
            path.push("llc");
            let mut res = std::process::Command::new(path.to_str().unwrap())
                .args(vec![program_bc_path.to_str().unwrap()])
                .output().unwrap();
            println!("{}", String::from_utf8(res.stdout).unwrap());
            eprintln!("{}", String::from_utf8(res.stderr).unwrap());
        }
        return Err((String::from_utf8(res.stdout).unwrap(), String::from_utf8(res.stderr).unwrap(), res.status.code().or(Some(255)).unwrap()));
    }
    Ok(out_path)
}



// println!("clang++ {:?}", args2);
// let res = std::process::Command::new("clang++")
//     .args(args2)
//     .output().unwrap();
//
// if res.status.code() != Some(0) {
//     eprintln!("clang++ failed to compile bitcode");
//     return (String::from_utf8(res.stdout).unwrap(), String::from_utf8(res.stderr).unwrap(), res.status.code().or(Some(255)).unwrap());
// }
//
// let args = vec![bitcode_out_path.to_str().unwrap(), runtime_out_path.to_str().unwrap(), "-o", out_path.to_str().unwrap()];
// println!("clang++ {:?}", args);
// let res = std::process::Command::new("clang++")
//     .args(args)
//     .output().unwrap();
//
// if res.status.code() != Some(0) {
//     eprintln!("clang++ failed to compile bitcode");
//     return (String::from_utf8(res.stdout).unwrap(), String::from_utf8(res.stderr).unwrap(), res.status.code().or(Some(255)).unwrap());
// }


// pub fn run_old(bitcode: MemoryBuffer) -> (String, String, i32) {
//     let temp_dir = TempDir::new("llvm-awk").unwrap();
//
//     let mut bitcode_path = temp_dir.path().join("awk.bc");
//     let mut asm_path = temp_dir.path().join("awk.asm");
//     let mut out_path = temp_dir.path().join("a.out");
//
//     {
//         let mut file = File::create(bitcode_path.clone()).unwrap();
//         file.write_all(bitcode.as_slice()).expect(&format!("could not write to {}", bitcode_path.to_str().unwrap()));
//     }
//
//     let res = std::process::Command::new("/Users/n8ta/llvm-13/bin/llc")
//         .args(vec![bitcode_path.to_str().unwrap(), "-o", asm_path.to_str().unwrap()])
//         .output().unwrap();
//
//     if res.status.code() != Some(0) {
//         eprintln!("llc failed to compile into assembly");
//         return (String::from_utf8(res.stdout).unwrap(), String::from_utf8(res.stderr).unwrap(), res.status.code().unwrap());
//     }
//
//     let res = std::process::Command::new("clang")
//         .args(vec![asm_path.to_str().unwrap(), dylib, "-o", out_path.to_str().unwrap()])
//         .output().unwrap();
//
//     if res.status.code() != Some(0) {
//         eprintln!("clang failed to link with runtime library");
//         return (String::from_utf8(res.stdout).unwrap(), String::from_utf8(res.stderr).unwrap(), res.status.code().unwrap());
//     }
//
//     let res = std::process::Command::new(out_path).output().unwrap();
//     return (String::from_utf8(res.stdout).unwrap(), String::from_utf8(res.stderr).unwrap(), res.status.code().unwrap());
// }