use std::ffi::CString;
use std::io::Write;

#[no_mangle]
pub extern "C" fn print_f64(number: f64) {
    println!("{}", number)
}