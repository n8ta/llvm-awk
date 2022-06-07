use tempfile::tempdir;
use crate::codgen;
use crate::parser::Stmt;

pub fn capture(stmt: Stmt, files: &[String]) -> (String, String, i32) {
    let path = tempdir().unwrap();
    let output_bc = path.path().join("output.bc");
    let output_exe = path.path().join("output.out");
    codgen::compile_to_bc(stmt, files, output_bc.clone());
    let args = vec!["-g", output_bc.to_str().unwrap(), "-o", output_exe.to_str().unwrap()];
    println!("clang++ {:?}", args);
    let res = std::process::Command::new("clang++")
        .args(args)
        .output().expect("to be able to link with clang");
    if res.status.code() != Some(0) {
        eprintln!("clang++ failed to compile bitcode");
        return (String::from_utf8(res.stdout).unwrap(), String::from_utf8(res.stderr).unwrap(), res.status.code().or(Some(255)).unwrap());
    }
    let res = std::process::Command::new(output_exe.to_str().unwrap())
        .output().expect("to be able to run the program");
    (String::from_utf8(res.stdout).unwrap(), String::from_utf8(res.stderr).unwrap(), res.status.code().or(Some(255)).unwrap())
}

//
//
// pub fn run(bitcode: MemoryBuffer, save_executable: Option<PathBuf>) {
//     let start = Instant::now();
//     let temp_dir = tempdir().unwrap();
//     println!("time to make temp dir: {}", start.elapsed().as_millis());
//
//     match external_tools(&temp_dir, bitcode, save_executable) {
//         Ok(out_path) => {
//             let start = Instant::now();
//             let mut child = std::process::Command::new(out_path)
//                 .spawn()
//                 .expect("to launch awk process");
//             child.wait().expect("for awk program to complete normally");
//             println!("time running final prog: {}", start.elapsed().as_millis());
//         }
//         Err(err) => {
//             println!("{}", err.0);
//             eprintln!("{}", err.1);
//         }
//     }
// }
//
// pub fn run_and_capture(bitcode: MemoryBuffer) -> (String, String, i32) {
//     let temp_dir = TempDir::new().unwrap();
//     match external_tools(&temp_dir, bitcode, None) {
//         Ok(out_path) => {
//             println!("out_path {:?}", out_path);
//             let res = std::process::Command::new(out_path).output().unwrap();
//             println!("res: {:?}", res);
//             return (String::from_utf8(res.stdout).expect("stdout"), String::from_utf8(res.stderr).expect("stderr"), res.status.code().or(Some(255)).unwrap());
//         }
//         Err(err) => return err,
//     }
// }
//
// pub fn external_tools(temp_dir: &TempDir, bitcode: MemoryBuffer, save_executable: Option<PathBuf>) -> Result<PathBuf, (String, String, i32)> {
//     let program_bc_path = temp_dir.path().join("awk.bc");
//     let runtime_bc_path = temp_dir.path().join("runtime.bc");
//     let out_path = if let Some(save) = save_executable { save } else { temp_dir.path().join("a.out") };
//
//     let start = Instant::now();
//     {
//         let mut file = File::create(program_bc_path.clone()).unwrap();
//         file.write_all(bitcode.as_slice()).expect(&format!("could not write to {}", program_bc_path.to_str().unwrap()));
//     }
//     {
//         let mut file = File::create(runtime_bc_path.clone()).unwrap();
//         file.write_all(RUNTIME_BITCODE).expect(&format!("could not write to {}", runtime_bc_path.to_str().unwrap()));
//     }
//     println!("time to write bc files: {}", start.elapsed().as_millis());
//
//     let args = vec!["-g", runtime_bc_path.to_str().unwrap(), program_bc_path.to_str().unwrap(), "-o", out_path.to_str().unwrap()];
//     println!("clang++ {:?}", args);
//
//     let start = Instant::now();
//     let res = std::process::Command::new("clang++")
//         .args(args)
//         .output().expect("to be able to link with clang");
//     println!("time to call clang: {}", start.elapsed().as_millis());
//
//
//     if res.status.code() != Some(0) {
//         eprintln!("clang++ failed to compile bitcode");
//         if let Ok(val) = std::env::var("LLVM_SYS_130_PREFIX") {
//             // llc gives helpful error messages for bitcode unlike clang. So if clang dies make
//             // an attempt to call LLC llc and print its explanation of the failure
//             let mut path = PathBuf::from(val);
//             path.push("bin");
//             path.push("llc");
//             let res = std::process::Command::new(path.to_str().unwrap())
//                 .args(vec![program_bc_path.to_str().unwrap()])
//                 .output().unwrap();
//             println!("{}", String::from_utf8(res.stdout).unwrap());
//             eprintln!("{}", String::from_utf8(res.stderr).unwrap());
//         }
//         return Err((String::from_utf8(res.stdout).unwrap(), String::from_utf8(res.stderr).unwrap(), res.status.code().or(Some(255)).unwrap()));
//     }
//     Ok(out_path)
// }
//
//
//
// // println!("clang++ {:?}", args2);
// // let res = std::process::Command::new("clang++")
// //     .args(args2)
// //     .output().unwrap();
// //
// // if res.status.code() != Some(0) {
// //     eprintln!("clang++ failed to compile bitcode");
// //     return (String::from_utf8(res.stdout).unwrap(), String::from_utf8(res.stderr).unwrap(), res.status.code().or(Some(255)).unwrap());
// // }
// //
// // let args = vec![bitcode_out_path.to_str().unwrap(), runtime_out_path.to_str().unwrap(), "-o", out_path.to_str().unwrap()];
// // println!("clang++ {:?}", args);
// // let res = std::process::Command::new("clang++")
// //     .args(args)
// //     .output().unwrap();
// //
// // if res.status.code() != Some(0) {
// //     eprintln!("clang++ failed to compile bitcode");
// //     return (String::from_utf8(res.stdout).unwrap(), String::from_utf8(res.stderr).unwrap(), res.status.code().or(Some(255)).unwrap());
// // }
//
//
// // pub fn run_old(bitcode: MemoryBuffer) -> (String, String, i32) {
// //     let temp_dir = TempDir::new("llvm-awk").unwrap();
// //
// //     let mut bitcode_path = temp_dir.path().join("awk.bc");
// //     let mut asm_path = temp_dir.path().join("awk.asm");
// //     let mut out_path = temp_dir.path().join("a.out");
// //
// //     {
// //         let mut file = File::create(bitcode_path.clone()).unwrap();
// //         file.write_all(bitcode.as_slice()).expect(&format!("could not write to {}", bitcode_path.to_str().unwrap()));
// //     }
// //
// //     let res = std::process::Command::new("/Users/n8ta/llvm-13/bin/llc")
// //         .args(vec![bitcode_path.to_str().unwrap(), "-o", asm_path.to_str().unwrap()])
// //         .output().unwrap();
// //
// //     if res.status.code() != Some(0) {
// //         eprintln!("llc failed to compile into assembly");
// //         return (String::from_utf8(res.stdout).unwrap(), String::from_utf8(res.stderr).unwrap(), res.status.code().unwrap());
// //     }
// //
// //     let res = std::process::Command::new("clang")
// //         .args(vec![asm_path.to_str().unwrap(), dylib, "-o", out_path.to_str().unwrap()])
// //         .output().unwrap();
// //
// //     if res.status.code() != Some(0) {
// //         eprintln!("clang failed to link with runtime library");
// //         return (String::from_utf8(res.stdout).unwrap(), String::from_utf8(res.stderr).unwrap(), res.status.code().unwrap());
// //     }
// //
// //     let res = std::process::Command::new(out_path).output().unwrap();
// //     return (String::from_utf8(res.stdout).unwrap(), String::from_utf8(res.stderr).unwrap(), res.status.code().unwrap());
// // }