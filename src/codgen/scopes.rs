use std::collections::HashMap;
use crate::codgen::ValueT;

pub type ScopeInfo<'ctx> = HashMap<String, ValueT<'ctx>>;

pub struct Scope<'ctx> {
    pub values: ScopeInfo<'ctx>,
}

pub struct Scopes<'ctx> {
    scopes: Vec<Scope<'ctx>>,
}

impl<'ctx> Scopes<'ctx> {
    pub fn new() -> Self {
        let scope = Scope { values: HashMap::default() };
        Scopes { scopes: vec![scope] }
    }
    pub fn insert(&mut self, name: String, value: ValueT<'ctx>) {
        self.scopes.last_mut().unwrap().values.insert(name, value);
    }
    #[allow(dead_code)]
    pub fn get(&self, name: &str) -> Option<&ValueT<'ctx>> {
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
    pub fn end_scope(&mut self) -> ScopeInfo<'ctx> {
        self.scopes.pop().unwrap().values
    }
    pub fn lookup(&self, name: &str) -> Option<ValueT<'ctx>> {
        for scope in self.scopes.iter().rev() {
            if let Some(val) = scope.values.get(name) {
                return Some(val.clone());
            }
        }
        None
    }
}
