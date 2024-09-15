use crate::expressions::Literal;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Environment {
    pub values: HashMap<String, Literal>,
    pub parent: Option<Box<Environment>>,
}

impl Environment {
    pub fn new(parent: Option<Box<Environment>>) -> Environment {
        Environment {
            values: HashMap::new(),
            parent,
        }
    }

    pub fn contains(&self, name: &str) -> bool {
        self.values.contains_key(name)
    }

    pub fn bind(&mut self, name: &str, value: Literal) {
        if self.contains(name) {
            self.values.insert(name.to_owned(), value);
        } else if let Some(ref mut parent) = self.parent {
            parent.bind(name, value);
        } else {
            self.values.insert(name.to_owned(), value);
        }
    }

    pub fn get(&self, name: &str) -> Option<&Literal> {
        if let Some(value) = self.values.get(name) {
            Some(value)
        } else if let Some(ref parent) = self.parent {
            parent.get(name)
        } else {
            None
        }
    }
}
