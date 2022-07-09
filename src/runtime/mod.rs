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

pub extern "C" fn print_string(_data: *mut c_void, value: *mut String) {
    println!("printing str {:?}", value);
    let str = unsafe { Box::from_raw(value) };
    if str.ends_with("\n") {
        print!("{}", str);
    } else {
        println!("{}", str);
    }
    Box::into_raw(str);
    println!("done printing str");
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

extern "C" fn column(data_ptr: *mut c_void, tag: u8, value: f64, pointer: *mut String) -> *mut String {
    let mut data = cast_to_runtime_data(data_ptr.clone());
    if data.full_line.is_none() {
        return Box::into_raw(Box::new("".to_string()))
    };
    let column = if tag == FLOAT_TAG {
        value.round() as i64
    } else if tag == STRING_TAG {
        let str = unsafe { Box::from_raw(pointer) };
        let number = str.parse::<f64>().expect(&format!("couldn't convert string to number {}", str));
        Box::into_raw(str);
        number.round() as i64
    } else {
        panic!("Called column with bad tag: {}", tag)
    };
    if column == 0 {
        let line = data.full_line.as_ref().expect("full line to be set").clone();
        return Box::into_raw(Box::new(line))
    }
    let line = data.full_line.as_ref().unwrap();
    for (idx, str_res) in line.split(data.RS).enumerate() {
        if idx + 1 == (column as usize) {
            let mut string = str_res.to_string();
            let ptr =Box::into_raw(Box::new(string.clone()));
            // println!("column returning {}, ptr {:?}", &string, &ptr);
            return ptr;
        }
    }
    return Box::into_raw(Box::new("".to_string()));
}

extern "C" fn free_string(data: *mut c_void, tag: u8, value: f64) -> f64 {
    let str = cast_float_to_string(value); // implicitly free'ed after this func
    0.0
}

extern "C" fn string_to_number(data: *mut c_void, tag: u8, value: f64) -> f64 {
    if tag == FLOAT_TAG {
        panic!("Tried to convert number to number????")
    }
    let string = cast_float_to_string(value);
    let number = string.parse::<f64>().expect(&format!("couldn't convert string to number {}", string));
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


pub struct Runtime {
    runtime_data: *mut c_void,
    pub next_line: *mut c_void,
    pub column: *mut c_void,
    pub free_string: *mut c_void,
    pub string_to_number: *mut c_void,
    pub number_to_string: *mut c_void,
    pub print_string: *mut c_void,
    pub print_float: *mut c_void,
    // pub is_truthy: *mut c_void,
}


const CANARY: &'static str = "runtime data loaded correctly";

pub struct RuntimeData {
    RS: char,
    FS: char,
    canary: String,
    files: Vec<String>,
    current_file: Option<Box<dyn BufRead>>,
    full_line: Option<String>,
    output: String,
}

impl RuntimeData {
    pub fn new(files: Vec<String>) -> RuntimeData {
        RuntimeData {
            files,
            canary: String::from(CANARY),
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
        let print_string = if capture_output { print_string_capture } else { print_string } as *mut c_void;
        let print_float = if capture_output { print_float_capture } else { print_float } as *mut c_void;
        Runtime {
            runtime_data: (ptr as *mut RuntimeData) as *mut c_void,
            next_line: next_line as *mut c_void,
            column: column as *mut c_void,
            free_string: free_string as *mut c_void,
            string_to_number: string_to_number as *mut c_void,
            number_to_string: number_to_string as *mut c_void,
            print_string,
            print_float,
        }
    }
    pub fn data_ptr(&self) -> *mut c_void {
        self.runtime_data
    }
    pub fn output(&self) -> String {
        cast_to_runtime_data(self.runtime_data).output.clone()
    }
}