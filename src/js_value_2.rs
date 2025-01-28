use core::panic;
use std::{
    cell::{OnceCell, RefCell},
    collections::HashMap,
    rc::Rc,
};

use crate::{error::EngineError, execution_engine::ExecutionContextRef};

enum JSValue {
    Undefined,
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    Object(JSObject),
}

type JSValueRef = &'static mut JSValue;

impl JSValue {
    fn as_object(&mut self) -> &mut JSObject {
        match self {
            JSValue::Object(obj) => obj,
            _ => panic!(),
        }
    }
}

thread_local! {
  static ALLOCATOR: OnceCell<Rc<RefCell<Allocator>>> = OnceCell::new();
}

struct Allocator {
    heap: HashMap<usize, *mut JSValue>,
}

impl Allocator {
    fn object() -> *mut JSValue {
        let object = Box::into_raw(Box::new(JSValue::Object(JSObject {
            call: None,
            constructor: None,
            properties: HashMap::new(),
            prototype: None,
        })));

        let allocator = Self::get_instance();
        allocator.borrow_mut().heap.insert(object.addr(), object);

        object
    }

    fn number(value: f64) -> *mut JSValue {
        let object = Box::into_raw(Box::new(JSValue::Number(value)));

        let allocator = Self::get_instance();
        allocator.borrow_mut().heap.insert(object.addr(), object);

        object
    }

    fn boolean(value: bool) -> *mut JSValue {
        let object = Box::into_raw(Box::new(JSValue::Boolean(value)));

        let allocator = Self::get_instance();
        allocator.borrow_mut().heap.insert(object.addr(), object);

        object
    }

    fn string(value: &str) -> *mut JSValue {
        let object = Box::into_raw(Box::new(JSValue::String(value.to_string())));

        let allocator = Self::get_instance();
        allocator.borrow_mut().heap.insert(object.addr(), object);

        object
    }

    fn null() -> *mut JSValue {
        let object = Box::into_raw(Box::new(JSValue::Null));

        let allocator = Self::get_instance();
        allocator.borrow_mut().heap.insert(object.addr(), object);

        object
    }

    fn undefined() -> *mut JSValue {
        let object = Box::into_raw(Box::new(JSValue::Undefined));

        let allocator = Self::get_instance();
        allocator.borrow_mut().heap.insert(object.addr(), object);

        object
    }

    fn deref(obj: *mut JSValue) -> &'static mut JSValue {
        if let Some(value) = Self::get_instance().borrow().heap.get(&obj.addr()) {
            unsafe {
                return value.as_mut().unwrap();
            }
        }

        panic!("Failed to deref");
    }

    fn get_instance() -> Rc<RefCell<Allocator>> {
        ALLOCATOR
            .try_with(|val| {
                val.get_or_init(|| {
                    Rc::new(RefCell::new(Allocator {
                        heap: HashMap::new(),
                    }))
                })
                .clone()
            })
            .unwrap()
    }
}

pub struct JSFunctionContext {
    pub ctx: ExecutionContextRef,
    pub js_args: Vec<*mut JSValue>,
    pub this: *mut JSValue,
}

struct JSObjectPropertyValue {
    is_enumarable: bool,
    value: *mut JSValue,
}

struct JSObject {
    prototype: Option<*mut JSValue>, // [[Prototype]]
    call: Option<fn(JSFunctionContext) -> Result<*mut JSValue, EngineError>>,
    constructor: Option<fn(JSFunctionContext) -> Result<*mut JSValue, EngineError>>,
    properties: HashMap<String, JSObjectPropertyValue>,
}

thread_local! {
  static OBJECT_GLOBAL: OnceCell<*mut JSValue>  = OnceCell::new();
}

impl JSObject {
    pub fn new_from_object_prototype() -> *mut JSValue {
        let obj = Allocator::object();
        Allocator::deref(obj).as_object().prototype = Some(Self::get_object_prototype());

        todo!()
    }

    fn get_object_instance() -> *mut JSValue {
        OBJECT_GLOBAL
            .try_with(|value| *value.get_or_init(|| Self::init_object_instance()))
            .unwrap()
    }

    fn get_object_prototype() -> *mut JSValue {
        Allocator::deref(Self::get_object_instance())
            .as_object()
            .prototype
            .unwrap()
    }

    fn init_object_instance() -> *mut JSValue {
        let object = Allocator::object();
        let prototype = Allocator::object();

        Allocator::deref(object).as_object().prototype = Some(prototype);

        object
    }
}

struct JSFunction {
    object: Rc<JSObject>,
}

thread_local! {
  static FUNCTION_GLOBAL: OnceCell<*mut JSValue>  = OnceCell::new();
}

impl JSFunction {
    pub fn new<F>(
        value: fn(JSFunctionContext) -> Result<*mut JSValue, EngineError>,
        name: Option<&str>,
    ) -> *mut JSValue {
        let obj = Allocator::object();

        Allocator::deref(obj).as_object().prototype = Some(Self::get_function_prototype());
        Allocator::deref(obj).as_object().call = Some(value);

        obj
    }

    fn get_function_instance() -> *mut JSValue {
        FUNCTION_GLOBAL
            .try_with(|value| *value.get_or_init(|| Self::init_function_instance()))
            .unwrap()
    }

    fn get_function_prototype() -> *mut JSValue {
        Allocator::deref(Self::get_function_instance())
            .as_object()
            .prototype
            .unwrap()
    }

    fn init_function_instance() -> *mut JSValue {
        let object_prototype = JSObject::get_object_prototype();
        let function_prototype = Allocator::object();
        Allocator::deref(function_prototype).as_object().prototype = Some(object_prototype);

        let function = Allocator::object();

        Allocator::deref(function).as_object().prototype = Some(function_prototype);

        function
    }
}
