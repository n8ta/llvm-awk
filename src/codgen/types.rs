use inkwell::module::{Linkage, Module};
use inkwell::types::StructType;
use inkwell::values::FunctionValue;

pub struct Types<'ctx> {
    pub value: StructType<'ctx>,
    pub print: FunctionValue<'ctx>,
    pub to_bool: FunctionValue<'ctx>,
}

impl<'ctx> Types<'ctx> {
    pub fn new(context: &'ctx inkwell::context::Context, module: &Module<'ctx>) -> Types<'ctx> {
        let i8 = context.i8_type();
        let i64 = context.i64_type();
        let value = context.struct_type(&[i8.into(), i64.into()], false);
        let print_value_type = context.void_type().fn_type(&[i8.into(), i64.into()], false);
        let to_bool_i64_type = context.i64_type().fn_type(&[i8.into(), i64.into()], false);
        let print = module.add_function("print_value", print_value_type, Some(Linkage::ExternalWeak));
        let to_bool = module.add_function("to_bool_i64", to_bool_i64_type, Some(Linkage::ExternalWeak));

        Types {
            value,
            print,
            to_bool,
        }
    }
}
