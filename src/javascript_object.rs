use std::{cell::RefCell, rc::Rc};

#[derive(Debug, Clone)]
pub enum JavascriptObjectKind {
    Number { value: f32 },
    String { value: String },
    Undefined,
    Boolean { value: bool },
}

#[derive(Debug)]
pub struct JavascriptObject {
    pub memory_id: u64,
    pub kind: JavascriptObjectKind,
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
            _ => String::from(""),
        }
    }

    pub fn cast_to_bool(&self) -> bool {
        match self.kind.clone() {
            JavascriptObjectKind::String { value } => !value.is_empty(),
            JavascriptObjectKind::Undefined => false,
            JavascriptObjectKind::Number { value } => value != 0.0,
            JavascriptObjectKind::Boolean { value } => value,
        }
    }

    pub fn is_equal_to_non_strict(&self, other_ref: &JavascriptObjectRef) -> bool {
        let b = other_ref.borrow();

        match self.kind.clone() {
            JavascriptObjectKind::String { value } => value == b.cast_to_string(),
            JavascriptObjectKind::Boolean { value } => value == b.cast_to_bool(),
            JavascriptObjectKind::Number { value } => value == b.cast_to_number(),
            JavascriptObjectKind::Undefined => b.is_undefined(),
        }
    }
}

pub type JavascriptObjectRef = Rc<RefCell<JavascriptObject>>;
