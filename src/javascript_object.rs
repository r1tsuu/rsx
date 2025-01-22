use core::fmt;
use std::{cell::RefCell, collections::HashMap, fmt::Debug, rc::Rc};

use crate::{error::EngineError, execution_engine::ExecutionEngine};

pub struct JavascriptFunctionContext<'a> {
    pub execution_engine: &'a mut ExecutionEngine,
    pub arguments: Vec<JavascriptObjectRef>,
    pub set_return_value: fn(JavascriptObjectRef),
    pub set_error: fn(&EngineError),
}

pub type JavascriptFunctionObjectValue = Rc<RefCell<dyn Fn(JavascriptFunctionContext)>>;

#[derive(Clone)]
pub enum JavascriptObjectKind {
    Number {
        value: f32,
    },
    String {
        value: String,
    },
    Undefined,
    Boolean {
        value: bool,
    },
    Function {
        value: JavascriptFunctionObjectValue,
    },
}

pub struct JavascriptObject {
    pub memory_id: u64,
    pub kind: JavascriptObjectKind,
}

impl fmt::Debug for JavascriptObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.create_debug_string())
    }
}

impl JavascriptObject {
    pub fn new(memory_id: u64, kind: JavascriptObjectKind) -> Self {
        JavascriptObject { memory_id, kind }
    }

    pub fn new_boolean(memory_id: u64, value: bool) -> Self {
        Self::new(memory_id, JavascriptObjectKind::Boolean { value })
    }

    pub fn new_number(memory_id: u64, value: f32) -> Self {
        Self::new(memory_id, JavascriptObjectKind::Number { value })
    }

    pub fn new_string(memory_id: u64, value: String) -> Self {
        Self::new(memory_id, JavascriptObjectKind::String { value })
    }

    pub fn new_undefined(memory_id: u64) -> Self {
        Self::new(memory_id, JavascriptObjectKind::Undefined)
    }

    pub fn new_function(memory_id: u64, value: JavascriptFunctionObjectValue) -> Self {
        Self::new(memory_id, JavascriptObjectKind::Function { value })
    }

    pub fn is_number(&self) -> bool {
        matches!(self.kind, JavascriptObjectKind::Number { .. })
    }

    pub fn is_string(&self) -> bool {
        matches!(self.kind, JavascriptObjectKind::String { .. })
    }

    pub fn is_boolean(&self) -> bool {
        matches!(self.kind, JavascriptObjectKind::Boolean { .. })
    }

    pub fn is_undefined(&self) -> bool {
        matches!(self.kind, JavascriptObjectKind::Undefined)
    }

    pub fn create_debug_string(&self) -> String {
        let type_val = match self.kind.clone() {
            JavascriptObjectKind::Boolean { .. } => "Boolean",
            JavascriptObjectKind::Function { .. } => "Function",
            JavascriptObjectKind::Number { .. } => "Number",
            JavascriptObjectKind::String { .. } => "String",
            JavascriptObjectKind::Undefined => "Undefined",
        };

        let str = format!(
            "JavascriptObject::{}, Address: 0x{:X}, Value: {}",
            type_val,
            self.memory_id,
            self.cast_to_string()
        );

        return str;
    }

    pub fn cast_to_number(&self) -> f32 {
        match self.kind.clone() {
            JavascriptObjectKind::Number { value } => value,
            JavascriptObjectKind::String { value } => value.parse::<f32>().unwrap_or(0.0),
            JavascriptObjectKind::Boolean { value } => {
                if value {
                    1.0
                } else {
                    0.0
                }
            }
            JavascriptObjectKind::Undefined => 0.0,
            JavascriptObjectKind::Function { .. } => 0.0,
        }
    }

    pub fn cast_to_string(&self) -> String {
        match self.kind.clone() {
            JavascriptObjectKind::Number { value } => value.to_string(),
            JavascriptObjectKind::Undefined => String::from("undefined"),
            JavascriptObjectKind::Boolean { value } => {
                if value {
                    String::from("true")
                } else {
                    String::from("false")
                }
            }
            JavascriptObjectKind::String { value } => value,
            JavascriptObjectKind::Function { .. } => String::from("Function"),
        }
    }

    pub fn cast_to_bool(&self) -> bool {
        match self.kind.clone() {
            JavascriptObjectKind::String { value } => !value.is_empty(),
            JavascriptObjectKind::Undefined => false,
            JavascriptObjectKind::Number { value } => value != 0.0,
            JavascriptObjectKind::Boolean { value } => value,
            JavascriptObjectKind::Function { .. } => true,
        }
    }

    pub fn is_equal_to_non_strict(&self, other_ref: &JavascriptObjectRef) -> bool {
        let b = other_ref.borrow();

        match self.kind.clone() {
            JavascriptObjectKind::String { value } => value == b.cast_to_string(),
            JavascriptObjectKind::Boolean { value } => value == b.cast_to_bool(),
            JavascriptObjectKind::Number { value } => value == b.cast_to_number(),
            JavascriptObjectKind::Undefined => b.is_undefined(),
            JavascriptObjectKind::Function { .. } => b.memory_id == self.memory_id,
        }
    }
}

pub type JavascriptObjectRef = Rc<RefCell<JavascriptObject>>;
