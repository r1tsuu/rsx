use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{error::EngineError, js_value::JSValueRef};

pub struct ExecutionScope {
    variables: RefCell<HashMap<String, JSValueRef>>,
    parent: Option<ExecutionScopeRef>,
}

pub type ExecutionScopeRef = Rc<ExecutionScope>;

impl ExecutionScope {
    pub fn new(parent: Option<ExecutionScopeRef>) -> ExecutionScopeRef {
        Rc::new(Self {
            variables: RefCell::new(HashMap::new()),
            parent,
        })
    }

    // Define a variable in the current scope
    pub fn define(&self, name: &str, object: JSValueRef) -> Result<JSValueRef, EngineError> {
        match self.get_current_scope_only(name) {
            None => {
                self.variables
                    .borrow_mut()
                    .insert(name.to_string(), object.clone());
                Ok(object)
            }
            Some(_) => Err(EngineError::execution_scope_error(format!(
                "Trying to define {} when already exists",
                name
            ))),
        }
    }

    // Get a variable, searching in the current scope and parent scopes
    pub fn get(&self, name: &str) -> Option<JSValueRef> {
        if let Some(value) = self.variables.borrow().get(name) {
            Some(value.clone())
        } else if let Some(ref parent) = self.parent {
            parent.get(name)
        } else {
            None
        }
    }

    // Get a variable, searching in the current scope
    pub fn get_current_scope_only(&self, name: &str) -> Option<JSValueRef> {
        if let Some(value) = self.variables.borrow().get(name) {
            Some(value.clone())
        } else {
            None
        }
    }

    // Assign a value to an existing variable, searching in parent scopes
    pub fn assign(&self, name: &str, object: JSValueRef) -> bool {
        if self.variables.borrow().contains_key(name) {
            self.variables.borrow_mut().insert(name.to_string(), object);
            true
        } else if let Some(ref parent) = self.parent {
            parent.assign(name, object)
        } else {
            false
        }
    }
}
