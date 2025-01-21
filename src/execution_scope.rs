use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{error::EngineError, javascript_object::JavascriptObjectRef, memory::MemoryRef};

#[derive(Debug)]
pub struct ExecutionScope {
    variables: HashMap<String, u64>,
    parent: Option<Rc<RefCell<ExecutionScope>>>,
    memory: MemoryRef,
}

impl ExecutionScope {
    pub fn new(parent: Option<Rc<RefCell<ExecutionScope>>>, memory: MemoryRef) -> Self {
        Self {
            variables: HashMap::new(),
            parent,
            memory,
        }
    }

    // Define a variable in the current scope
    pub fn define(
        &mut self,
        name: String,
        object: JavascriptObjectRef,
    ) -> Result<JavascriptObjectRef, EngineError> {
        match self.get_current_scope_only(name.clone()) {
            None => {
                self.variables.insert(name, object.borrow().memory_id);
                Ok(object)
            }
            Some(_) => Err(EngineError::execution_scope_error(format!(
                "Trying to define {} when already exists",
                name
            ))),
        }
    }

    // Get a variable, searching in the current scope and parent scopes
    pub fn get(&self, name: String) -> Option<JavascriptObjectRef> {
        if let Some(value) = self.variables.get(&name) {
            Some(self.memory.borrow().get_by_id(*value).unwrap())
        } else if let Some(ref parent) = self.parent {
            parent.borrow().get(name)
        } else {
            None
        }
    }

    // Get a variable, searching in the current scope
    pub fn get_current_scope_only(&self, name: String) -> Option<JavascriptObjectRef> {
        if let Some(value) = self.variables.get(&name) {
            Some(self.memory.borrow().get_by_id(*value).unwrap())
        } else {
            None
        }
    }

    // Assign a value to an existing variable, searching in parent scopes
    pub fn assign(&mut self, name: String, object: JavascriptObjectRef) -> bool {
        if self.variables.contains_key(&name) {
            self.variables
                .insert(name.to_string(), object.borrow().memory_id);
            true
        } else if let Some(ref parent) = self.parent {
            parent.borrow_mut().assign(name, object)
        } else {
            false
        }
    }

    pub fn get_variable_ids(&self) -> Vec<u64> {
        Vec::from_iter(self.variables.iter().map(|entry| *entry.1))
    }
}
