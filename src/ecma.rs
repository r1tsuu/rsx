/**
 * ECMA Standard Library Modules
 * Like: Object, Function, Array, Math, Map, Set, etc.
 */
pub trait JSModule {
    fn name(&self) -> &str;
    fn init(&mut self, vm: &mut VM);
}

use std::rc::Rc;

use crate::{
    ast::FunctionDefinitionExpression,
    error::EngineError,
    vm::{CallContext, JSValue, NativeFunction, Object, ObjectRef, Scope, VM},
};

pub const PROTOTYPE: &'static str = "prototype";

pub const OBJECT: &'static str = "Object";

pub struct ObjectClass {}

impl JSModule for ObjectClass {
    fn name(&self) -> &str {
        OBJECT
    }

    fn init(&mut self, vm: &mut VM) {
        let prototype = Object::new().alloc(vm);

        let constructor = Object::new()
            .with_property(PROTOTYPE, JSValue::from_object_ref(prototype.clone()))
            .alloc(vm);

        prototype
            .load_mut(vm)
            .set_property("constructor", JSValue::from_object_ref(constructor.clone()));

        vm.global_this
            .load_mut(vm)
            .set_prototype(prototype.clone()) // set global object's prototype
            .set_property(OBJECT, JSValue::from_object_ref(constructor.clone()));
    }
}

pub const OBJECT_STRING: &'static str = "[object Object]";

impl ObjectClass {
    pub fn new() -> impl JSModule {
        Self {}
    }

    pub fn create(vm: &mut VM) -> Object {
        Object::new().with_prototype(Self::prototype(vm))
    }

    pub fn str_fallback() -> String {
        OBJECT_STRING.to_string()
    }

    pub fn prototype(vm: &mut VM) -> ObjectRef {
        vm.global_constructor_prototype(OBJECT)
            .expect("Called prototype before Object init")
    }

    fn init_methods(vm: &mut VM, function_prototype: ObjectRef, object_prototype: ObjectRef) {
        let func = JSValue::native_function(function_prototype.clone(), Self::to_string, vm);
        object_prototype.load_mut(vm).set_property("toString", func);
    }

    fn to_string(_: &mut VM, _: CallContext) -> Result<JSValue, EngineError> {
        Ok(JSValue::string(OBJECT_STRING))
    }
}

const FUNCTION: &str = "Function";

pub struct FunctionClass {}

impl JSModule for FunctionClass {
    fn name(&self) -> &str {
        FUNCTION
    }

    fn init(&mut self, vm: &mut VM) {
        let object_prototype = ObjectClass::prototype(vm);

        let prototype = Object::new()
            .with_prototype(object_prototype.clone())
            .alloc(vm);

        let to_string = JSValue::native_function(prototype.clone(), Self::to_string, vm);
        prototype.load_mut(vm).set_property("toString", to_string);

        ObjectClass::init_methods(vm, prototype.clone(), object_prototype);

        let constructor = Object::new()
            .with_property(PROTOTYPE, JSValue::from_object_ref(prototype.clone()))
            .alloc(vm);

        prototype
            .load_mut(vm)
            .set_property("constructor", JSValue::from_object_ref(constructor.clone()));

        vm.global_this
            .load_mut(vm)
            .set_property(FUNCTION, JSValue::from_object_ref(constructor.clone()));
    }
}

impl FunctionClass {
    pub fn new() -> impl JSModule {
        Self {}
    }

    pub fn create_native(vm: &mut VM, call: NativeFunction) -> Object {
        Object::new()
            .with_prototype(Self::prototype(vm))
            .with_call_native(call)
            .with_captured_scope(vm.scopes.len() - 1)
    }

    pub fn create_from_ast(vm: &mut VM, ast: FunctionDefinitionExpression) -> Object {
        let index = vm.function_definitions.len();
        vm.function_definitions.push(Rc::new(ast));

        Object::new()
            .with_prototype(Self::prototype(vm))
            .with_call_ast(index)
            .with_captured_scope(vm.scopes.len() - 1)
    }

    pub fn prototype(vm: &mut VM) -> ObjectRef {
        vm.global_constructor_prototype(FUNCTION)
            .expect("Called prototype before Function init")
    }

    fn to_string(_vm: &mut VM, _call: CallContext) -> Result<JSValue, EngineError> {
        Ok(JSValue::string("function () { [native code] }"))
    }
}

const ARRAY: &str = "Array";

pub struct ArrayClass {}

impl JSModule for ArrayClass {
    fn name(&self) -> &str {
        ARRAY
    }

    fn init(&mut self, vm: &mut VM) {
        let prototype = Object::new()
            .with_prototype(ObjectClass::prototype(vm))
            .with_property("length", JSValue::Number(0.0))
            .with_property(
                "push",
                JSValue::native_function(FunctionClass::prototype(vm), Self::push, vm),
            )
            .with_property(
                "pop",
                JSValue::native_function(FunctionClass::prototype(vm), Self::pop, vm),
            )
            .alloc(vm);

        let constructor = Object::new()
            .with_property(PROTOTYPE, JSValue::from_object_ref(prototype.clone()))
            .with_prototype(FunctionClass::prototype(vm))
            .alloc(vm);

        prototype
            .load_mut(vm)
            .set_property("constructor", JSValue::from_object_ref(constructor.clone()));

        vm.global_this
            .load_mut(vm)
            .set_property(ARRAY, JSValue::from_object_ref(constructor.clone()));
    }
}

impl ArrayClass {
    pub fn new() -> impl JSModule {
        Self {}
    }

    pub fn prototype(vm: &mut VM) -> ObjectRef {
        vm.global_constructor_prototype(ARRAY)
            .expect("Called prototype before Array init")
    }

    pub fn create(vm: &mut VM) -> Object {
        Object::new()
            .with_prototype(Self::prototype(vm))
            .with_property("length", JSValue::Number(0.0))
    }

    pub fn push(vm: &mut VM, call: CallContext) -> Result<JSValue, EngineError> {
        let mut length = {
            call.this
                .load_mut(vm)
                .get_property("length")
                .and_then(|property| property.try_as_number())
                .expect("Array.length is not a number") as usize
        };

        for arg in call.args.iter() {
            call.this
                .load_mut(vm)
                .set_property(&length.to_string(), arg.clone());

            length += 1;

            call.this
                .load_mut(vm)
                .set_property("length", JSValue::Number(length as f32));
        }

        Ok(call.args.last().cloned().unwrap_or(JSValue::Undefined))
    }

    pub fn pop(vm: &mut VM, call: CallContext) -> Result<JSValue, EngineError> {
        let mut length = {
            call.this
                .load(vm)
                .get_property("length")
                .and_then(|property| property.try_as_number())
                .expect("Array.length is not a number") as usize
        };

        if length == 0 {
            return Ok(JSValue::Undefined);
        }

        length -= 1;

        let value = call
            .this
            .load(vm)
            .get_property(&length.to_string())
            .unwrap_or(JSValue::Undefined);

        call.this
            .load_mut(vm)
            .delete_property(&length.to_string())
            .set_property("length", JSValue::Number(length as f32));

        Ok(value)
    }
}
