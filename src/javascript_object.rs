use std::{cell::RefCell, rc::Rc};

#[derive(Debug, Clone)]
pub enum JavascriptObjectKind {
    Number { value: f32 },
    String { value: String },
    Undefined,
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

    pub fn is_undefined(&self) -> bool {
        matches!(self.kind, JavascriptObjectKind::Undefined)
    }

    pub fn cast_to_number(&self) -> f32 {
        match self.kind {
            JavascriptObjectKind::Number { value } => value,
            _ => 0.0,
        }
    }

    pub fn cast_to_string(&self) -> String {
        match self.kind.clone() {
            JavascriptObjectKind::String { value } => value,
            _ => String::from(""),
        }
    }
}

pub type JavascriptObjectRef = Rc<RefCell<JavascriptObject>>;
