use std::ffi::CString;
use std::io::Write;

#[no_mangle]
pub extern "C" fn print_f64(number: f64) {
    println!("{}", number)
}

#[repr(C)]
#[derive(Debug)]
pub struct Value {
    tag: i8,
    value: [u8; 8],
}

#[no_mangle]
pub extern "C" fn to_bool_i64(value: Value) -> i64 {
    println!("to_bool_i64 {:?}", value);
    if value.tag == 0 {
        let i64_val = i64::from_le_bytes(value.value);
        if i64_val == 0 { 0 as i64} else { 1 as i64 }
    } else {
        let f64_val = f64::from_le_bytes(value.value);
        if f64_val == 0.0 { 0 as i64 } else { 1 as i64 }
    }
}