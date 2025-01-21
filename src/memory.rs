use std::{cell::RefCell, collections::HashMap, rc::Rc, vec};

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

    pub fn get_by_id(&self, id: u64) -> Option<JavascriptObjectRef> {
        self.heap.get(&id).cloned()
    }

    pub fn deallocate_except_ids(&mut self, ids: &[u64]) {
        let mut keys_to_remove = Vec::new();

        for (memory_id, _) in self.heap.iter_mut() {
            if !ids.contains(memory_id) {
                keys_to_remove.push(*memory_id);
            }
        }

        for memory_id in keys_to_remove {
            self.heap.remove(&memory_id);
        }
    }
}
