mod casting;

use std::ffi::c_void;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use crate::codgen::{FLOAT_TAG, STRING_TAG};
use crate::runtime::casting::{cast_float_to_string, cast_str_to_float, cast_to_runtime_data};

fn get_next_file(data: &mut RuntimeData) -> bool {
    if let Some(next_file) = data.files.pop() {
        let next_file = match File::open(PathBuf::from(next_file.clone())) {
            Ok(f) => f,
            Err(err) => panic!("Failed to open file '{}'", next_file)
        };
        data.current_file = Some(Box::new(BufReader::new(next_file)));
        true
    } else {
        false
    }
}

pub extern "C" fn print_value(_data: *mut c_void, tag: u8, value: f64) -> f64 {
    if tag == FLOAT_TAG {
        println!("{}", value);
    } else if tag == STRING_TAG {
        let string = cast_float_to_string(value);
        if string.ends_with("\n") {
            print!("{}", string);
        } else {
            println!("{}", string);
        }
        Box::leak(string);
    } else {
        panic!("print_value called w/bad tag {}", tag);
    }
    0.0
}

pub extern "C" fn print_value_capture(data: *mut c_void, tag: u8, value: f64) -> f64 {
    let data = cast_to_runtime_data(data);
    let output = if tag == FLOAT_TAG {
        format!("{}", value)
    } else if tag == STRING_TAG {
        let string = cast_float_to_string(value);
        format!("{}", string)
    } else {
        panic!("print_value called w/bad tag {}", tag);
    };
    println!("{}", output);
    data.output.push_str(&output);
    if !data.output.ends_with("\n") {
        data.output.push('\n');
    }
    0.0
}

extern "C" fn next_line(data: *mut c_void) -> f64 {
    // println!("next line called");
    let mut data = cast_to_runtime_data(data);
    if data.current_file.is_none() {
        if !get_next_file(data) {
            return 0.0;
        }
    }
    loop {
        let file = data.current_file.as_mut().unwrap();
        let mut result = String::new();
        match file.read_line(&mut result) {
            Ok(_) => {}
            Err(err) => {
                panic!("Failed to read from file. Error: {}", err)
            }
        }
        // println!("read line: {}", result);
        if result.len() == 0 {
            if !get_next_file(data) {
                return 0.0;
            }
        } else {
            // println!("Assigning full line");
            data.full_line = Some(result);
            break;
        }
    }
    1.0
}

extern "C" fn column(data_ptr: *mut c_void, tag: u8, value: f64) -> f64 {
    // println!("column called {}", value);
    let mut data = cast_to_runtime_data(data_ptr.clone());
    if data.full_line.is_none() {
        return cast_str_to_float(Box::new("".to_string()));
    };
    let column = if tag == FLOAT_TAG {
        value.round() as i64
    } else if tag == STRING_TAG {
        let num = string_to_number(data_ptr, tag, value);
        num.round() as i64
    } else {
        panic!("Called column with bad tag: {}", tag)
    };
    if column == 0 {
        let line = data.full_line.as_ref().expect("full line to be set").clone();
        return cast_str_to_float(Box::new(line));
    }
    let line = data.full_line.as_ref().unwrap();
    for (idx, str_res) in line.split(data.RS).enumerate() {
        if idx + 1 == (column as usize) {
            let mut string = str_res.to_string();
            return cast_str_to_float(Box::new(string));
        }
    }
    return cast_str_to_float(Box::new("".to_string()));
}

extern "C" fn free_string(data: *mut c_void, tag: u8, value: f64) -> f64 {
    let str = cast_float_to_string(value); // implicitly free'ed after this func
    0.0
}

extern "C" fn is_truthy(data: *mut c_void, tag: u8, value: f64) -> i32 {
    println!("is truthy {} {}", tag, value);
    if tag == FLOAT_TAG {
        if value != 0.0 { 1 } else { 0 }
    } else if tag == STRING_TAG {
        let str = cast_float_to_string(value);
        let length = str.len();
        Box::leak(str);
        if length != 0 { 1 } else { 0 }
    } else {
        panic!("bad tag detected in is_truthy runtime func: {} {}", tag, value);
    }
}


extern "C" fn string_to_number(data: *mut c_void, tag: u8, value: f64) -> f64 {
    if tag == FLOAT_TAG {
        panic!("Tried to convert number to number????")
    }
    let string = cast_float_to_string(value);
    let number=  string.parse::<f64>().expect(&format!("couldn't convert string to number {}", string));
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


pub extern "C" fn hello_world(runtime_data: *mut c_void) {
    let runtime_data = cast_to_runtime_data(runtime_data);
    println!("hello from rust!");
}

pub struct Runtime {
    runtime_data: *mut c_void,
    pub next_line: *mut c_void,
    pub column: *mut c_void,
    pub free_string: *mut c_void,
    pub string_to_number: *mut c_void,
    pub number_to_string: *mut c_void,
    pub hello_world: *mut c_void,
    pub print_value: *mut c_void,
    pub is_truthy: *mut c_void,
}

pub struct RuntimeData {
    RS: char,
    FS: char,
    files: Vec<String>,
    current_file: Option<Box<dyn BufRead>>,
    full_line: Option<String>,
    output: String,
}

impl RuntimeData {
    pub fn new(files: Vec<String>) -> RuntimeData {
        RuntimeData {
            files,
            current_file: None,
            full_line: None,
            RS: ' ',
            FS: '\n',
            output: String::new(),
        }
    }
}

impl Runtime {
    pub fn new(files: Vec<String>, capture_output: bool) -> Runtime {
        let data = Box::new(RuntimeData::new(files));
        let ptr = Box::leak(data);
        let print_value = if capture_output { print_value_capture } else {
            print_value
        } as *mut c_void;
        Runtime {
            runtime_data: (ptr as *mut RuntimeData) as *mut c_void,
            next_line: next_line as *mut c_void,
            column: column as *mut c_void,
            free_string: free_string as *mut c_void,
            string_to_number: string_to_number as *mut c_void,
            number_to_string: number_to_string as *mut c_void,
            is_truthy: is_truthy as *mut c_void,
            print_value,
            hello_world: hello_world as *mut c_void,
        }
    }
    pub fn data_ptr(&self) -> *mut c_void {
        self.runtime_data
    }
    pub fn output(&self) -> String {
        cast_to_runtime_data(self.runtime_data).output.clone()
    }
}