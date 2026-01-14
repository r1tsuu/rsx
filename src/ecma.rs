use crate::core::{ExecutionContext, Object, ObjectRef};

pub trait JSModule<'exec> {
    fn name() -> &'static str;
    fn create(exec: &'exec mut ExecutionContext<'exec>) -> Self;
}

pub trait JSClass<'exec> {
    fn name(&self) -> &str;
    fn prototype(&mut self, exec: &'exec mut ExecutionContext<'exec>) -> ObjectRef<'exec>;
    fn constructor(&mut self, exec: &'exec mut ExecutionContext<'exec>) -> ObjectRef<'exec>;
}

pub struct ObjectClass<'exec> {
    _prototype: Option<ObjectRef<'exec>>,
    _constructor: Option<ObjectRef<'exec>>,
}

impl<'exec> JSClass<'exec> for ObjectClass<'exec> {
    fn name(&self) -> &str {
        return "Object";
    }

    fn prototype(&mut self, exec: &'exec mut ExecutionContext<'exec>) -> ObjectRef<'exec> {
        self._prototype.clone().unwrap_or_else(|| {
            let _prototype = Object::new().build();
            self._prototype = Some(_prototype.clone());
            _prototype
        })
    }

    fn constructor(&mut self, exec: &'exec mut ExecutionContext<'exec>) -> ObjectRef<'exec> {
        self._constructor.clone().unwrap_or_else(|| {
            let _constructor = Object::new().build();
            self._constructor = Some(_constructor.clone());

            exec.subscribe_to_class_initialization("Function", move |class, exec| {
                let proto = class.prototype(exec);

                _constructor.borrow_mut().set_prototype(proto.clone());
            });

            _constructor
        })
    }
}
