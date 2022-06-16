use std::collections::HashMap;
use crate::codgen::{ValuePtrT, ValueT};

pub type ScopeInfo = HashMap<String, ValueT>;

pub struct Scope {
    pub values: ScopeInfo,
}

pub struct Scopes {
    scopes: Vec<Scope>,
}

impl Scopes {
    pub fn new() -> Self {
        let scope = Scope { values: HashMap::default() };
        Scopes { scopes: vec![scope] }
    }
    pub fn insert(&mut self, name: String, value: ValuePtrT) {
        self.scopes.last_mut().unwrap().values.insert(name, value);
    }
    #[allow(dead_code)]
    pub fn get(&self, name: &str) -> Option<&ValuePtrT> {
        for scope in self.scopes.iter().rev() {
            if let Some(val) = scope.values.get(name) {
                return Some(val);
            }
        }
        None
    }
    pub fn begin_scope(&mut self) {
        self.scopes.push(Scope { values: HashMap::default() });
    }
    pub fn end_scope(&mut self) -> ScopeInfo {
        self.scopes.pop().unwrap().values
    }
    pub fn lookup(&self, name: &str) -> Option<ValuePtrT> {
        for scope in self.scopes.iter().rev() {
            if let Some(val) = scope.values.get(name) {
                return Some(val.clone());
            }
        }
        None
    }
}
