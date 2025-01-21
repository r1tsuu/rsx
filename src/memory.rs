use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::javascript_object::{JavascriptObject, JavascriptObjectRef};

pub struct Memory {
    heap: HashMap<u64, JavascriptObjectRef>,
    next_id: u64,
}

impl Memory {
    pub fn new() -> Self {
        Memory {
            heap: HashMap::new(),
            next_id: 0,
        }
    }

    pub fn allocate(&mut self, obj: JavascriptObject) -> JavascriptObjectRef {
        let rc_object = Rc::new(RefCell::new(obj));
        self.heap.insert(self.next_id, rc_object.clone());
        self.next_id += 1;
        rc_object
    }

    pub fn allocate_undefined(&mut self) -> JavascriptObjectRef {
        self.allocate(JavascriptObject::new_undefined(self.next_id))
    }

    pub fn allocate_number(&mut self, value: f32) -> JavascriptObjectRef {
        self.allocate(JavascriptObject::new_number(self.next_id, value))
    }

    pub fn allocate_string(&mut self, value: String) -> JavascriptObjectRef {
        self.allocate(JavascriptObject::new_string(self.next_id, value))
    }
}
