use std::{cell::RefCell, rc::Rc};

#[derive(Debug)]
pub enum JavascriptObject {
    Number { value: f32 },
    String { value: String },
    Undefined,
}

impl JavascriptObject {
    pub fn is_number(&self) -> bool {
        matches!(self, JavascriptObject::Number { .. })
    }

    pub fn is_string(&self) -> bool {
        matches!(self, JavascriptObject::String { .. })
    }

    pub fn is_undefined(&self) -> bool {
        matches!(self, JavascriptObject::Undefined)
    }

    pub fn cast_to_number(&self) -> f32 {
        match self {
            JavascriptObject::Number { value } => *value,
            _ => 0.0,
        }
    }

    pub fn cast_to_string(&self) -> String {
        match self {
            JavascriptObject::String { value } => value.clone(),
            _ => String::from(""),
        }
    }
}

pub type JavascriptObjectRef = Rc<RefCell<JavascriptObject>>;
