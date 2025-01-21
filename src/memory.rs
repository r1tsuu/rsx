use std::{cell::RefCell, collections::HashMap, rc::Rc, vec};

use crate::javascript_object::{JavascriptObject, JavascriptObjectRef};

pub struct Memory {
    heap: Vec<JavascriptObjectRef>,
}

impl Memory {
    pub fn new() -> Self {
        Memory { heap: vec![] }
    }

    pub fn allocate(&mut self, obj: JavascriptObject) -> JavascriptObjectRef {
        let rc_object = Rc::new(RefCell::new(obj));
        self.heap.push(rc_object.clone());
        rc_object
    }

    pub fn allocate_undefined(&mut self) -> JavascriptObjectRef {
        self.allocate(JavascriptObject::Undefined)
    }

    pub fn allocate_number(&mut self, value: f32) -> JavascriptObjectRef {
        self.allocate(JavascriptObject::Number { value })
    }

    pub fn allocate_string(&mut self, value: String) -> JavascriptObjectRef {
        self.allocate(JavascriptObject::String { value })
    }
}
