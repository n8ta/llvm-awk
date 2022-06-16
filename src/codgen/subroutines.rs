// use inkwell::builder::Builder;
// use inkwell::context::Context;
// use inkwell::IntPredicate;
// use inkwell::module::Module;
// use inkwell::values::FunctionValue;
// use crate::codgen::runtime::Runtime;
//
// pub struct Subroutines<'ctx> {
//     pub free_if_string: FunctionValue<'ctx>,
// }
//
// impl<'ctx> Subroutines<'ctx> {
//     pub fn new(context: &'ctx Context, module: &Module<'ctx>, runtime: &Runtime<'ctx>, builder: &mut Builder) -> Self {
//         let free_if_string = Subroutines::build_free_if_string(context, module, runtime, builder);
//         Subroutines {
//             free_if_string
//         }
//     }
//     fn build_free_if_string(context: &'ctx Context, module: &Module<'ctx>, runtime: &Runtime<'ctx>, builder: &mut Builder) -> FunctionValue<'ctx> {
//         let ffi_type = &[context.i8_type().into(), context.f64_type().into()];
//
//         let free_if_string = module.add_function("free_if_string", context.void_type().fn_type(ffi_type, false), None);
//         let init_bb = context.append_basic_block(free_if_string, "free_string_init");
//         let free_string_bb = context.append_basic_block(free_if_string, "free_string_bb");
//         let ret_bb = context.append_basic_block(free_if_string, "ret_bb");
//
//         // init -> free_string_bb or ret_bb
//         builder.position_at_end(init_bb);
//         let tag = free_if_string.get_nth_param(0).expect("to have 1 tag arg").into_int_value();
//         let value = free_if_string.get_nth_param(1).expect("to have 1 value arg").into_float_value();
//         let tag_cmp = builder.build_int_compare(IntPredicate::EQ, tag, context.i8_type().const_int(1, false), "cmp_with_str_tag");
//         builder.build_conditional_branch(tag_cmp, free_string_bb, ret_bb);
//
//         // free the string
//         builder.position_at_end(free_string_bb);
//         builder.build_call(runtime.free_string, &[tag.into(), value.into()], "call_free");
//         builder.build_return(None);
//
//         // ret
//         builder.position_at_end(ret_bb);
//         builder.build_return(None);
//
//         free_if_string
//     }
// }
//
