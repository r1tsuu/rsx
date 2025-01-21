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
}

pub type JavascriptObjectRef = Rc<RefCell<JavascriptObject>>;
