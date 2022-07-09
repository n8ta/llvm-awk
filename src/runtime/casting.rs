use std::os::raw::c_void;
use crate::runtime::{CANARY, RuntimeData};

pub fn cast_float_to_string(value: f64) -> Box<String> {
    unsafe {
        let transmute_to_ptr: *mut u8 = { std::mem::transmute(value) };
        println!("ptr is {:?}", transmute_to_ptr);
        let str_ptr = transmute_to_ptr as *mut String;
        let str: Box<String> = Box::from_raw(str_ptr);
        str
    }
}
pub fn cast_str_to_float(mut value: Box<String>) -> f64 {
    let ptr = &mut *value as *mut String;
    let res = unsafe { std::mem::transmute::<*mut String, f64>(ptr) };
    Box::leak(value);
    res
}


pub fn cast_to_runtime_data(data: *mut c_void) -> &'static mut RuntimeData {
    unsafe {
        let data = data as *mut RuntimeData;
        let d = &mut *data;
        if d.canary != CANARY {
            eprintln!("RUNTIME DATA LOADED WRONG. CANARY MISSING");
            std::process::exit(-1);
        }
        d
    }
}

#[test]
fn test_casting() {
    let expected = Box::new(format!("TEST!"));

    let input = Box::new(format!("TEST!"));
    let float_ptr = cast_str_to_float(input);
    let i64_ptr: i64 = unsafe { std::mem::transmute(float_ptr) };
    let string = cast_float_to_string(float_ptr);
    assert_eq!(string, expected);
}

#[test]
fn test_casting_mut() {
    let expected = Box::new(format!("foo-bar"));

    let input = Box::new("foo".to_string());
    let float_ptr = cast_str_to_float(input);
    let mut string = cast_float_to_string(float_ptr);
    assert_eq!(string, Box::new("foo".to_string()));

    string.push('-');
    string.push('b');
    string.push('a');
    string.push('r');
    assert_eq!(string, expected);

    let float_ptr = cast_str_to_float(string);
    let string = cast_float_to_string(float_ptr);
    assert_eq!(string, expected);
}
