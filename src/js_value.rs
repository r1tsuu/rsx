use core::fmt;
use std::{cell::RefCell, rc::Rc};

use crate::execution_engine::ExecutionContextRef;

pub struct JSFunctionArgs {
    pub ctx: ExecutionContextRef,
    pub js_args: Vec<JSValueRef>,
}

pub type JSFunctionValue = Rc<dyn Fn(JSFunctionArgs)>;

#[derive(Clone)]
pub enum JSValueKind {
    Number { value: f32 },
    String { value: String },
    Undefined,
    Boolean { value: bool },
    Function { value: JSFunctionValue },
}

pub struct JSValue {
    pub kind: JSValueKind,
}

pub type JSValueRef = Rc<JSValue>;

impl fmt::Debug for JSValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.create_debug_string())
    }
}

impl JSValue {
    pub fn new(kind: JSValueKind) -> JSValueRef {
        Rc::new(Self { kind })
    }

    pub fn new_boolean(value: bool) -> JSValueRef {
        Self::new(JSValueKind::Boolean { value })
    }

    pub fn new_number(value: f32) -> JSValueRef {
        Self::new(JSValueKind::Number { value })
    }

    pub fn new_string(value: &str) -> JSValueRef {
        Self::new(JSValueKind::String {
            value: value.to_string(),
        })
    }

    pub fn new_undefined() -> JSValueRef {
        Self::new(JSValueKind::Undefined)
    }

    pub fn new_function<F>(value: F) -> JSValueRef
    where
        F: Fn(JSFunctionArgs) + 'static,
    {
        // Assuming Self::new is a constructor that takes a JSValueKind enum variant.
        Self::new(JSValueKind::Function {
            value: Rc::new(value),
        })
    }

    pub fn is_number(&self) -> bool {
        matches!(self.kind, JSValueKind::Number { .. })
    }

    pub fn is_string(&self) -> bool {
        matches!(self.kind, JSValueKind::String { .. })
    }

    pub fn is_boolean(&self) -> bool {
        matches!(self.kind, JSValueKind::Boolean { .. })
    }

    pub fn is_function(&self) -> bool {
        matches!(self.kind, JSValueKind::Function { .. })
    }

    pub fn is_undefined(&self) -> bool {
        matches!(self.kind, JSValueKind::Undefined)
    }

    pub fn addr(&self) -> usize {
        let ptr = self as *const JSValue;
        ptr.addr()
    }

    pub fn create_debug_string(&self) -> String {
        let type_val = match &self.kind {
            JSValueKind::Boolean { .. } => "Boolean",
            JSValueKind::Function { .. } => "Function",
            JSValueKind::Number { .. } => "Number",
            JSValueKind::String { .. } => "String",
            JSValueKind::Undefined => "Undefined",
        };

        let str = format!(
            "JavascriptObject::{}, Address: 0x{:X}, Value: {}",
            type_val,
            self.addr(),
            self.cast_to_string()
        );

        return str;
    }

    pub fn cast_to_number(&self) -> f32 {
        match &self.kind {
            JSValueKind::Number { value } => *value,
            JSValueKind::String { value } => value.parse::<f32>().unwrap_or(0.0),
            JSValueKind::Boolean { value } => {
                if *value {
                    1.0
                } else {
                    0.0
                }
            }
            JSValueKind::Undefined => 0.0,
            JSValueKind::Function { .. } => 0.0,
        }
    }

    pub fn cast_to_string(&self) -> String {
        match &self.kind {
            JSValueKind::Number { value } => value.to_string(),
            JSValueKind::Undefined => "undefined".to_string(),
            JSValueKind::Boolean { value } => {
                if *value {
                    "true".to_string()
                } else {
                    "false".to_string()
                }
            }
            JSValueKind::String { value } => value.clone(),
            JSValueKind::Function { .. } => "Function".to_string(),
        }
    }

    pub fn cast_to_bool(&self) -> bool {
        match &self.kind {
            JSValueKind::String { value } => !value.is_empty(),
            JSValueKind::Undefined => false,
            JSValueKind::Number { value } => *value != 0.0,
            JSValueKind::Boolean { value } => *value,
            JSValueKind::Function { .. } => true,
        }
    }

    pub fn is_equal_to_non_strict(&self, other: &JSValueRef) -> bool {
        match &self.kind {
            JSValueKind::String { value } => *value == other.cast_to_string(),
            JSValueKind::Boolean { value } => *value == other.cast_to_bool(),
            JSValueKind::Number { value } => *value == other.cast_to_number(),
            JSValueKind::Undefined => other.is_undefined(),
            JSValueKind::Function { .. } => other.addr() == self.addr(),
        }
    }
}
