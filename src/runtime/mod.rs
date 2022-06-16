pub extern "C" fn hello_world() {
    println!("hello from rust!");
}

pub struct Runtime {
    hello_world: *mut std::os::raw::c_void,
    runtime_data: *mut RuntimeData,
}
struct RuntimeData {
    data: String,
}

impl Runtime {
    pub fn new() -> Runtime {
        let runtime_data = RuntimeData;
        Runtime {
            hello_world: hello_world as *mut std::os::raw::c_void
        }
    }
}