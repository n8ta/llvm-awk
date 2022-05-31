use std::os::unix::raw::mode_t;
use inkwell::AddressSpace;
use inkwell::module::{Linkage, Module};
use inkwell::types::StructType;
use inkwell::values::FunctionValue;

pub struct Types<'ctx> {
    pub value: StructType<'ctx>,
    pub print: FunctionValue<'ctx>,
    pub to_bool: FunctionValue<'ctx>,
    pub mismatch: FunctionValue<'ctx>,
    pub get_float: FunctionValue<'ctx>,
    pub next_line: FunctionValue<'ctx>,
    pub column: FunctionValue<'ctx>,
    pub add_file: FunctionValue<'ctx>,
    pub init: FunctionValue<'ctx>,
}

impl<'ctx> Types<'ctx> {
    pub fn new(context: &'ctx inkwell::context::Context, module: &Module<'ctx>) -> Types<'ctx> {
        // let i8 = context.i8_type();
        // let i64 = context.i64_type();
        // let print = runtime_mod.get_function("print_value").expect("to find print_value function in lib");
        // let to_bool= runtime_mod.get_function("to_bool_i64").expect("to find to_bool_i64 function in lib");
        // let mismatch = runtime_mod.get_function("print_mismatch").expect("to find print_mismatch function in lib");
        // let get_float = runtime_mod.get_function("get_float").expect("to find get_float function in lib");
        // let next_line = runtime_mod.get_function("next_line").expect("to find next_line function in lib");
        // let column = runtime_mod.get_function("column").expect("to find column function in lib");
        // let value = context.struct_type(&[i8.into(), i64.into()], false);
        //
        // Types {
        //     value,
        //     print,
        //     to_bool,
        //     mismatch,
        //     get_float,
        //     next_line,
        //     column
        // }


        let i8 = context.i8_type();
        let i64 = context.i64_type();
        let f64_type = context.f64_type();
        let value = context.struct_type(&[i8.into(), i64.into()], false);
        let print_value_type = context.void_type().fn_type(&[i8.into(), i64.into()], false);
        let mismatch_type = context.void_type().fn_type(&[], false);
        let get_float_type = f64_type.fn_type(&[], false);
        let to_bool_i64_type = context.i64_type().fn_type(&[i8.into(), i64.into()], false);
        let next_line_type = context.i64_type().fn_type(&[], false);
        let column_type = context.i64_type().fn_type(&[i8.into(), i64.into()], false);
        let mut message = String::new(); // to satisfy llvm types just pass everything as a 1000 long vector
        for i in 0..1000 {
            message.push('1');
        }
        let const_str = context.const_string(message.as_bytes(), true);
        let const_str_type= const_str.get_type();
        let ptr_to_const_str_type = const_str_type.ptr_type(AddressSpace::Generic).into();
        println!("{:?}", ptr_to_const_str_type);
        let add_file_type = context.void_type().fn_type(&[ptr_to_const_str_type], false);

        let print = module.add_function("print_value", print_value_type, Some(Linkage::ExternalWeak));
        let to_bool = module.add_function("to_bool_i64", to_bool_i64_type, Some(Linkage::ExternalWeak));
        let mismatch = module.add_function("print_mismatch", mismatch_type, Some(Linkage::ExternalWeak));
        let get_float = module.add_function("get_float", get_float_type, Some(Linkage::ExternalWeak));
        let next_line = module.add_function("next_line", next_line_type, Some(Linkage::ExternalWeak));
        let column = module.add_function("column", column_type, Some(Linkage::ExternalWeak));
        let add_file = module.add_function("add_file", add_file_type, Some(Linkage::ExternalWeak));
        let init = module.add_function("init", context.void_type().fn_type(&[], false), Some(Linkage::ExternalWeak));
        Types {
            value,
            print,
            to_bool,
            mismatch,
            get_float,
            next_line,
            column,
            init,
            add_file
        }
    }
}

pub fn pad(path: &mut String) {
    if path.as_bytes().len() > 1000 { panic!("file path is too long {}", path)}
    while path.len() < 1000 {
        path.push('\0');
    }
}
