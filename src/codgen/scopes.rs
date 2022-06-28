use std::collections::HashMap;
use crate::codgen::{ValuePtrT};

pub struct Scopes {
    scopes: HashMap<String, ValuePtrT>
}

impl Scopes {
    pub fn new() -> Self {
        Scopes { scopes: HashMap::default() }
    }
    pub fn insert(&mut self, name: String, value: ValuePtrT) {
        self.scopes.insert(name, value);
    }

    pub fn get(&self, name: &str) -> &ValuePtrT {
        self.scopes.get(name).unwrap()
    }
}
