mod live;
mod casting;
mod testing;
mod call_log;

use std::ffi::c_void;
pub use live::LiveRuntime;
pub use testing::TestRuntime;

pub(crate) trait Runtime {
    fn new(files: Vec<String>) -> Self;

    fn next_line(&self) -> *mut c_void;
    fn column(&self) -> *mut c_void;
    fn free_string(&self) -> *mut c_void;
    fn string_to_number(&self) -> *mut c_void;
    fn copy_string(&self) -> *mut c_void;
    fn number_to_string(&self) -> *mut c_void;
    fn print_string(&self) -> *mut c_void;
    fn print_float(&self) -> *mut c_void;
    fn data_ptr(&self) -> *mut c_void;
}