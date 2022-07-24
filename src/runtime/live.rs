use std::ffi::c_void;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use crate::codgen::{FLOAT_TAG, STRING_TAG};
use crate::columns::Columns;
use crate::runtime::Runtime;

pub const CANARY: &str = "this is the canary!";


pub extern "C" fn print_string(_data: *mut c_void, value: *mut String) {
    let str = unsafe { Box::from_raw(value) };
    if str.ends_with("\n") {
        print!("{}", str);
    } else {
        println!("{}", str);
    }
    Box::into_raw(str);
}

pub extern "C" fn print_float(_data: *mut c_void, value: f64) {
    println!("{}", value);
}

pub extern "C" fn print_string_capture(data: *mut c_void, value: *mut String) {
    let data = cast_to_runtime_data(data);
    let str = unsafe { Box::from_raw(value) };


    let res = if str.ends_with("\n") {
        format!("{}", str)
    } else {
        format!("{}\n", str)
    };
    data.output.push_str(&res);
    Box::into_raw(str);
}

pub extern "C" fn print_float_capture(data: *mut c_void, value: f64) {
    let data = cast_to_runtime_data(data);
    let res = format!("{}\n", value);
    data.output.push_str(&res);
}

extern "C" fn next_line(data: *mut c_void) -> f64 {
    let data = cast_to_runtime_data(data);
    if data.columns.next_line() { 1.0 } else { 0.0 }
}

extern "C" fn column(data_ptr: *mut c_void, tag: u8, value: f64, pointer: *mut String) -> *mut String {
    let data = cast_to_runtime_data(data_ptr);
    let idx =
        if tag == FLOAT_TAG {
            value
        } else {
            string_to_number(data_ptr, pointer)
        };
    let idx = idx.round() as usize;
    Box::into_raw(Box::new(data.columns.get(idx)))
}

extern "C" fn free_string(data: *mut c_void, string: *mut String) -> f64 {
    let string = unsafe { Box::from_raw(string) };
    0.0
}

extern "C" fn string_to_number(data: *mut c_void, ptr: *mut String) -> f64 {
    let string = unsafe { Box::from_raw(ptr) };
    let number: f64 = string.parse().expect(&format!("couldn't convert string to number {}", string));
    Box::leak(string);
    number
}

extern "C" fn number_to_string(data: *mut c_void, tag: u8, value: f64) -> f64 {
    if tag != FLOAT_TAG {
        panic!("Tried to convert non-number to string")
    }
    let value = if value.fract() == 0.0 {
        value.floor()
    } else { value };
    let heap_alloc_string = Box::new(value.to_string());
    let ptr = heap_alloc_string.as_ptr();
    Box::leak(heap_alloc_string);
    return unsafe { std::mem::transmute(ptr) };
}

extern "C" fn copy_string(data: *mut c_void, ptr: *mut c_void) -> *mut String {
    let original: Box<String> = unsafe { Box::from_raw(ptr as *mut String) };
    let copy = original.clone();
    Box::into_raw(original);
    Box::into_raw(copy) as *mut String
}



pub struct LiveRuntime {
    runtime_data: *mut c_void,
    pub next_line: *mut c_void,
    pub column: *mut c_void,
    pub free_string: *mut c_void,
    pub string_to_number: *mut c_void,
    pub number_to_string: *mut c_void,
    pub print_string: *mut c_void,
    pub print_float: *mut c_void,
    pub copy_string: *mut c_void,
}



pub struct RuntimeData {
    columns: Columns,
    canary: String,
    output: String,
}

impl RuntimeData {
    pub fn new(files: Vec<String>) -> RuntimeData {
        RuntimeData {
            canary: String::from(CANARY),
            columns: Columns::new(files),
            output: String::new(),
        }
    }
}

impl LiveRuntime {
    pub fn data_ptr(&self) -> *mut c_void {
        self.runtime_data
    }
    pub fn output(&self) -> String {
        cast_to_runtime_data(self.runtime_data).output.clone()
    }
}

impl Runtime for LiveRuntime {
    fn new(files: Vec<String>) -> LiveRuntime {
        let data = Box::new(RuntimeData::new(files));
        let ptr = Box::leak(data);
        LiveRuntime {
            runtime_data: (ptr as *mut RuntimeData) as *mut c_void,
            next_line: next_line as *mut c_void,
            column: column as *mut c_void,
            free_string: free_string as *mut c_void,
            string_to_number: string_to_number as *mut c_void,
            copy_string: copy_string as *mut c_void,
            number_to_string: number_to_string as *mut c_void,
            print_string: print_string as *mut c_void,
            print_float: print_float as *mut c_void,
        }
    }
    fn next_line(&self) -> *mut c_void {
        self.next_line
    }

    fn column(&self) -> *mut c_void {
        self.column
    }

    fn free_string(&self) -> *mut c_void {
        self.free_string
    }

    fn string_to_number(&self) -> *mut c_void {
        self.string_to_number
    }

    fn copy_string(&self) -> *mut c_void {
        self.copy_string
    }

    fn number_to_string(&self) -> *mut c_void {
        self.number_to_string
    }

    fn print_string(&self) -> *mut c_void {
        self.print_string
    }

    fn print_float(&self) -> *mut c_void {
        self.print_float
    }

    fn data_ptr(&self) -> *mut c_void {
        self.runtime_data
    }
}

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
