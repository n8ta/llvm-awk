use std::fmt::{Display, Formatter, write};
use std::io::Write;


#[repr(C)]
#[derive(Debug)]
pub struct Value {
    tag: i8,
    value: [u8; 8],
}

#[derive(Debug)]
enum ValueRusty {
    Int(i64),
    Float(f64),
}

impl ValueRusty {
    pub fn new(tag: i8, value: i64)-> Self {
        match tag {
            0 => {
                ValueRusty::Int(value)
            },
            1 => {
                let float = unsafe { std::mem::transmute::<i64, f64>(value) };
                ValueRusty::Float(float)
            }
            _ => {
                panic!("Unknown value {:?}", value);
            }
        }
    }
}

impl Display for ValueRusty {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ValueRusty::Int(val) => write!(f, "{}", val),
            ValueRusty::Float(val) => write!(f, "{}", val),
        }
    }
}

#[no_mangle]
pub extern "C" fn print_value(tag: i8, value: i64) {
    println!("tag:{} i64:{} float:{} bytes:{:?}", tag, value,  f64::from_le_bytes(value.to_le_bytes()), value.to_le_bytes());
    println!("{}", ValueRusty::new(tag, value))
}

#[no_mangle]
pub extern "C" fn print_mismatch() {
    eprintln!("Tried to mix float and integer operations!");
}

#[no_mangle]
pub extern "C" fn get_float() -> f64 {
    return 1.1;
}


#[no_mangle]
pub extern "C" fn to_bool_i64(tag: i8, value: i64) -> i64 {
    let value = ValueRusty::new(tag, value);
    let res = match value {
        ValueRusty::Int(val) => val == 0,
        ValueRusty::Float(val) => val == 0.0,
    };
    if res { 0 } else { 1 }
}