use crate::core::{Object, ObjectRef};
use std::sync::OnceLock;

pub struct ObjectClass<'exec> {
    _prototype: Option<ObjectRef<'exec>>,
    _constructor: Option<ObjectRef<'exec>>,
}

static OBJECT_CLASS: OnceLock<ObjectClass<'static>> = OnceLock::new();

impl<'exec> ObjectClass<'exec> {
    fn name() -> &'static str {
        "Object"
    }

    fn instance() -> &'static ObjectClass<'static> {
        OBJECT_CLASS.get_or_init(|| ObjectClass {
            _prototype: None,
            _constructor: None,
        })
    }

    pub fn prototype() -> ObjectRef<'exec> {
        let instance = Self::instance();

        instance
            ._prototype
            .unwrap_or_else(|| {
                let prototype = Object::new().build();
                instance._prototype = Some(prototype.clone());
                prototype
            })
            .clone()
    }

    pub fn constructor() -> ObjectRef<'exec> {
        let instance = Self::instance();

        instance
            ._constructor
            .unwrap_or_else(|| {
                let constructor = Object::new().build();
                instance._constructor = Some(constructor.clone());
                constructor
            })
            .clone()
    }

    pub fn init(function_prototype: ObjectRef<'exec>) {
        let instance = Self::instance();

        instance._constructor.inspect(|constructor| {
            constructor.borrow_mut().set_prototype(function_prototype);
        });
    }
}

pub struct FunctionClass<'exec> {
    _prototype: Option<ObjectRef<'exec>>,
    _constructor: Option<ObjectRef<'exec>>,
}

static FUNCTION_CLASS: OnceLock<FunctionClass<'static>> = OnceLock::new();

impl<'exec> FunctionClass<'exec> {
    fn name() -> &'static str {
        "Function"
    }

    fn instance() -> &'static FunctionClass<'static> {
        FUNCTION_CLASS.get_or_init(|| FunctionClass {
            _prototype: None,
            _constructor: None,
        })
    }

    pub fn prototype() -> ObjectRef<'exec> {
        let instance = Self::instance();

        instance
            ._prototype
            .unwrap_or_else(|| {
                let prototype = Object::new().build();
                instance._prototype = Some(prototype.clone());
                prototype
            })
            .clone()
    }

    pub fn constructor() -> ObjectRef<'exec> {
        let instance = Self::instance();

        instance
            ._constructor
            .unwrap_or_else(|| {
                let constructor = Object::new().with_prototype(Self::prototype()).build();
                instance._constructor = Some(constructor.clone());
                constructor
            })
            .clone()
    }

    pub fn init(object_prototype: ObjectRef<'exec>) {
        let instance = Self::instance();
        instance._constructor.inspect(|constructor| {
            constructor.borrow_mut().set_prototype(object_prototype);
        });
    }
}
