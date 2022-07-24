
#[derive(Clone, Debug)]
pub enum Call {
    NextLine,
    Column(f64, String),
    FreeString,
    StringToNumber,
    CopyString,
    NumberToString,
    PrintString,
    PrintFloat,
}

pub struct CallLog {
    pub log: Vec<Call>,
}

impl CallLog {
    pub fn new() -> Self { CallLog { log: vec![] } }
    pub fn log(&mut self, call: Call) {
        println!("call: {:?}", call);
        self.log.push(call)
    }
}