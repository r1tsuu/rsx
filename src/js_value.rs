use as_any::{AsAny, Downcast};

use core::f64;
use std::{
    any::Any,
    cell::{OnceCell, RefCell},
    collections::HashMap,
    fmt,
    rc::Rc,
};

use crate::execution_engine::ExecutionContextRef;

pub type JSValueRef = Rc<dyn JSValue>;

pub trait JSValue: AsAny {
    fn as_any_rc(self: Rc<Self>) -> Rc<dyn Any>;
    fn get_typeof(&self) -> JSStringRef;
    fn cast_to_number(&self) -> JSNumberRef;
    fn cast_to_string(&self) -> JSStringRef;
    fn cast_to_boolean(&self) -> JSBooleanRef;
    fn get_debug_string(&self) -> String;
    fn is_equal_to_non_strict(&self, other: &dyn JSValue) -> bool;

    fn add(&self, _other: &dyn JSValue) -> JSValueRef {
        JSNumber::get_nan()
    }

    fn divide(&self, _other: &dyn JSValue) -> JSValueRef {
        JSNumber::get_nan()
    }

    fn substract(&self, _other: &dyn JSValue) -> JSValueRef {
        JSNumber::get_nan()
    }

    fn multiply(&self, _other: &dyn JSValue) -> JSValueRef {
        JSNumber::get_nan()
    }
}

impl fmt::Debug for dyn JSValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.get_debug_string())
    }
}

#[derive(Clone)]
pub struct JSNumber {
    pub value: f64,
}

thread_local! {
  static NAN: OnceCell<JSNumberRef> = OnceCell::new();
}

pub type JSNumberRef = Rc<JSNumber>;

static NAN_NAME: &str = "NaN";

impl JSNumber {
    pub fn new(value: f64) -> JSNumberRef {
        if value.is_nan() {
            JSNumber::get_nan()
        } else {
            Rc::new(JSNumber { value })
        }
    }

    pub fn cast(value: &dyn JSValue) -> Option<&JSNumber> {
        value.downcast_ref::<JSNumber>()
    }

    pub fn cast_rc(value: &JSValueRef) -> Option<JSNumberRef> {
        value.clone().as_any_rc().downcast().ok()
    }

    pub fn get_nan() -> JSNumberRef {
        NAN.with(|value| {
            value
                .get_or_init(|| {
                    return Rc::new(JSNumber { value: f64::NAN });
                })
                .clone()
        })
    }

    pub fn get_nan_name() -> &'static str {
        NAN_NAME
    }
}

impl JSValue for JSNumber {
    fn as_any_rc(self: Rc<Self>) -> Rc<dyn Any> {
        self
    }

    fn get_typeof(&self) -> JSStringRef {
        JSString::new("number")
    }

    fn add(&self, other: &dyn JSValue) -> JSValueRef {
        if let Some(other) = JSNumber::cast(other) {
            return JSNumber::new(self.value + other.value);
        }

        if let Some(other) = JSBoolean::cast(other) {
            return JSNumber::new(self.value + other.as_f64());
        }

        if let Some(other) = JSString::cast(other) {
            return JSString::new(&format!("{}{}", self.cast_to_string().value, other.value));
        }

        JSNumber::get_nan()
    }

    fn substract(&self, other: &dyn JSValue) -> JSValueRef {
        if let Some(other) = JSNumber::cast(other) {
            return JSNumber::new(self.value - other.value);
        } else if let Some(other) = JSBoolean::cast(other) {
            return JSNumber::new(self.value - other.as_f64());
        }

        return JSNumber::get_nan();
    }

    fn divide(&self, other: &dyn JSValue) -> JSValueRef {
        if let Some(other) = JSNumber::cast(other) {
            return JSNumber::new(self.value / other.value);
        } else if let Some(other) = JSBoolean::cast(other) {
            return JSNumber::new(self.value / other.as_f64());
        }

        return JSNumber::get_nan();
    }

    fn multiply(&self, other: &dyn JSValue) -> JSValueRef {
        if let Some(other) = JSNumber::cast(other) {
            return JSNumber::new(self.value * other.value);
        } else if let Some(other) = JSBoolean::cast(other) {
            return JSNumber::new(self.value * other.as_f64());
        }

        return JSNumber::get_nan();
    }

    fn is_equal_to_non_strict(&self, other: &dyn JSValue) -> bool {
        self.value == other.cast_to_number().value
    }

    fn cast_to_number(&self) -> JSNumberRef {
        JSNumber::new(self.value)
    }

    fn cast_to_string(&self) -> JSStringRef {
        match self.value {
            v if v.is_nan() => JSString::new(NAN_NAME),
            _ => JSString::new(&self.value.to_string()),
        }
    }

    fn cast_to_boolean(&self) -> JSBooleanRef {
        match self.value {
            v if v.is_nan() => JSBoolean::get_false(),
            0.0 => JSBoolean::get_false(),
            _ => JSBoolean::get_true(),
        }
    }

    fn get_debug_string(&self) -> String {
        format!("Number {}", self.cast_to_string().value)
    }
}

pub struct JSString {
    pub value: String,
}

pub type JSStringRef = Rc<JSString>;

impl JSString {
    pub fn new(value: &str) -> JSStringRef {
        Rc::new(JSString {
            value: value.to_string(),
        })
    }

    pub fn cast(value: &dyn JSValue) -> Option<&JSString> {
        value.downcast_ref::<JSString>()
    }
}

impl JSValue for JSString {
    fn as_any_rc(self: Rc<Self>) -> Rc<dyn Any> {
        self
    }

    fn get_typeof(&self) -> JSStringRef {
        JSString::new("string")
    }

    fn add(&self, other: &dyn JSValue) -> JSValueRef {
        JSString::new(&format!("{}{}", self.value, other.cast_to_string().value))
    }

    fn is_equal_to_non_strict(&self, other: &dyn JSValue) -> bool {
        self.value == other.cast_to_string().value
    }

    fn cast_to_number(&self) -> JSNumberRef {
        self.value
            .parse::<f64>()
            .map(|val| JSNumber::new(val))
            .unwrap_or(JSNumber::get_nan())
    }

    fn cast_to_string(&self) -> JSStringRef {
        JSString::new(&self.value)
    }

    fn cast_to_boolean(&self) -> JSBooleanRef {
        if self.value.len() == 0 {
            JSBoolean::get_false()
        } else {
            JSBoolean::get_true()
        }
    }

    fn get_debug_string(&self) -> String {
        format!("String {}", self.value)
    }
}

pub struct JSBoolean {
    pub value: bool,
}

pub type JSBooleanRef = Rc<JSBoolean>;

thread_local! {
  static TRUE: OnceCell<JSBooleanRef> = OnceCell::new();
  static FALSE: OnceCell<JSBooleanRef> = OnceCell::new();
}

static TRUE_NAME: &str = "true";
static FALSE_NAME: &str = "false";

impl JSBoolean {
    pub fn get(value: bool) -> JSBooleanRef {
        if value {
            JSBoolean::get_true()
        } else {
            JSBoolean::get_false()
        }
    }

    pub fn get_true() -> JSBooleanRef {
        TRUE.with(|value| {
            value
                .get_or_init(|| {
                    return Rc::new(JSBoolean { value: true });
                })
                .clone()
        })
    }

    pub fn get_true_name() -> &'static str {
        TRUE_NAME
    }

    pub fn get_false() -> JSBooleanRef {
        FALSE.with(|value| {
            value
                .get_or_init(|| {
                    return Rc::new(JSBoolean { value: false });
                })
                .clone()
        })
    }

    pub fn get_false_name() -> &'static str {
        FALSE_NAME
    }

    pub fn as_f64(&self) -> f64 {
        if self.value {
            1.0
        } else {
            0.0
        }
    }

    pub fn cast(value: &dyn JSValue) -> Option<&JSBoolean> {
        value.downcast_ref::<JSBoolean>()
    }
}

impl JSValue for JSBoolean {
    fn as_any_rc(self: Rc<Self>) -> Rc<dyn Any> {
        self
    }

    fn get_typeof(&self) -> JSStringRef {
        JSString::new("boolean")
    }

    fn add(&self, other: &dyn JSValue) -> JSValueRef {
        JSNumber::add(&self.cast_to_number(), other)
    }

    fn divide(&self, other: &dyn JSValue) -> JSValueRef {
        JSNumber::divide(&self.cast_to_number(), other)
    }

    fn multiply(&self, other: &dyn JSValue) -> JSValueRef {
        JSNumber::multiply(&self.cast_to_number(), other)
    }

    fn substract(&self, other: &dyn JSValue) -> JSValueRef {
        JSNumber::substract(&self.cast_to_number(), other)
    }

    fn is_equal_to_non_strict(&self, other: &dyn JSValue) -> bool {
        self.value == other.cast_to_boolean().value
    }

    fn cast_to_number(&self) -> JSNumberRef {
        if self.value {
            JSNumber::new(1.0)
        } else {
            JSNumber::new(0.0)
        }
    }

    fn cast_to_string(&self) -> JSStringRef {
        if self.value {
            JSString::new(TRUE_NAME)
        } else {
            JSString::new(FALSE_NAME)
        }
    }

    fn cast_to_boolean(&self) -> JSBooleanRef {
        JSBoolean::get(self.value)
    }

    fn get_debug_string(&self) -> String {
        format!("Boolean {}", self.cast_to_string().value)
    }
}

pub struct JSUndefined;

pub type JSUndefinedRef = Rc<JSUndefined>;

thread_local! {
  static UNDEFINED: OnceCell<JSUndefinedRef> = OnceCell::new();
}

static UNDEFINED_NAME: &str = "undefined";

impl JSUndefined {
    pub fn get() -> JSUndefinedRef {
        UNDEFINED.with(|value| {
            value
                .get_or_init(|| {
                    return Rc::new(JSUndefined);
                })
                .clone()
        })
    }

    pub fn is(value: &dyn JSValue) -> bool {
        value.is::<JSUndefined>()
    }

    pub fn get_name() -> &'static str {
        UNDEFINED_NAME
    }
}

impl JSValue for JSUndefined {
    fn as_any_rc(self: Rc<Self>) -> Rc<dyn Any> {
        self
    }

    fn get_typeof(&self) -> JSStringRef {
        JSString::new("undefined")
    }

    fn cast_to_boolean(&self) -> JSBooleanRef {
        JSBoolean::get_false()
    }

    fn is_equal_to_non_strict(&self, other: &dyn JSValue) -> bool {
        JSUndefined::is(other)
    }

    fn cast_to_number(&self) -> JSNumberRef {
        JSNumber::get_nan()
    }

    fn cast_to_string(&self) -> JSStringRef {
        JSString::new(UNDEFINED_NAME)
    }

    fn get_debug_string(&self) -> String {
        "Undefined".to_string()
    }
}

pub struct JSNull;

pub type JSNullRef = Rc<JSNull>;

thread_local! {
  static NULL: OnceCell<JSNullRef> = OnceCell::new();
}

static NULL_NAME: &str = "null";

impl JSNull {
    pub fn get() -> JSNullRef {
        NULL.with(|value| {
            value
                .get_or_init(|| {
                    return Rc::new(JSNull);
                })
                .clone()
        })
    }

    pub fn is(value: &dyn JSValue) -> bool {
        value.is::<JSNull>()
    }

    pub fn get_name() -> &'static str {
        NULL_NAME
    }
}

impl JSValue for JSNull {
    fn as_any_rc(self: Rc<Self>) -> Rc<dyn Any> {
        self
    }

    fn get_typeof(&self) -> JSStringRef {
        JSString::new("object")
    }

    fn cast_to_boolean(&self) -> JSBooleanRef {
        JSBoolean::get_false()
    }

    fn cast_to_number(&self) -> JSNumberRef {
        JSNumber::get_nan()
    }

    fn is_equal_to_non_strict(&self, other: &dyn JSValue) -> bool {
        JSNull::is(other)
    }

    fn cast_to_string(&self) -> JSStringRef {
        JSString::new(NULL_NAME)
    }

    fn get_debug_string(&self) -> String {
        "Null".to_string()
    }
}

pub struct JSFunctionContext {
    pub ctx: ExecutionContextRef,
    pub js_args: Vec<JSValueRef>,
    pub this: JSObjectRef,
}

impl JSFunctionContext {
    pub fn arg(&self, index: usize) -> JSValueRef {
        self.js_args
            .get(index)
            .cloned()
            .unwrap_or(JSUndefined::get())
    }

    pub fn set_return(&self, value: JSValueRef) {
        self.ctx.set_current_function_return(value.clone());
    }
}

pub type JSFunctionValue = Rc<dyn Fn(JSFunctionContext)>;

pub struct JSFunction {
    pub value: JSFunctionValue,
    pub name: Option<String>,
}

pub type JSFunctionRef = Rc<JSFunction>;

impl JSFunction {
    pub fn new<F>(value: F, name: Option<&str>) -> JSValueRef
    where
        F: Fn(JSFunctionContext) + 'static,
    {
        Rc::new(JSFunction {
            value: Rc::new(value),
            name: name.map(str::to_string),
        })
    }

    pub fn new_named<F>(value: F, name: &str) -> JSValueRef
    where
        F: Fn(JSFunctionContext) + 'static,
    {
        JSFunction::new(value, Some(name))
    }

    pub fn cast(value: &dyn JSValue) -> Option<&JSFunction> {
        value.downcast_ref::<JSFunction>()
    }
}

impl JSValue for JSFunction {
    fn as_any_rc(self: Rc<Self>) -> Rc<dyn Any> {
        self
    }

    fn get_typeof(&self) -> JSStringRef {
        JSString::new("function")
    }

    fn cast_to_number(&self) -> JSNumberRef {
        JSNumber::get_nan()
    }

    fn is_equal_to_non_strict(&self, other: &dyn JSValue) -> bool {
        if let Some(other) = JSFunction::cast(other) {
            (self as *const JSFunction) == (other as *const JSFunction)
        } else {
            false
        }
    }

    fn cast_to_string(&self) -> JSStringRef {
        JSString::new("Function")
    }

    fn cast_to_boolean(&self) -> JSBooleanRef {
        JSBoolean::get_true()
    }

    fn get_debug_string(&self) -> String {
        let mut str = "Function".to_string();

        if let Some(name) = &self.name {
            str.push_str(&format!(" {}", name));
        } else {
            str.push_str(" Anonymous");
        }

        str
    }
}

pub struct JSObjectValue {
    value: JSValueRef,
    is_enumarable: bool,
}

impl JSObjectValue {
    pub fn new_value(value: JSValueRef) -> JSObjectValue {
        JSObjectValue {
            is_enumarable: true,
            value,
        }
    }
}

pub struct JSObject {
    pub value: RefCell<HashMap<String, JSObjectValue>>,
    pub prototype: Option<JSObjectRef>,
}

thread_local! {
  static OBJECT_PROTOTYPE: OnceCell<JSObjectRef> = OnceCell::new();
}

fn define_object_prototype() -> JSObjectRef {
    let prototype = JSObject::new();

    prototype.set_key_method("hasOwnProperty", |ctx| {
        let key = &ctx.arg(0).cast_to_string().value;
        ctx.set_return(JSBoolean::get(ctx.this.value.borrow().contains_key(key)));
    });

    prototype.set_key_method("toString", |ctx| {
        ctx.set_return(JSString::new("[object Object]"));
    });

    prototype.set_key_method("isPrototypeOf", |ctx| {
        let prototype = &ctx.arg(0);

        if let Some(arg_prototype) = JSObject::cast_rc(prototype.clone()) {
            if let Some(this_prototype) = ctx.this.prototype.clone() {
                ctx.set_return(JSBoolean::get(Rc::ptr_eq(&arg_prototype, &this_prototype)));
                return;
            }
        }

        ctx.set_return(JSBoolean::get_false());
    });

    prototype.set_key_method("valueOf", |ctx| {
        ctx.set_return(ctx.this.clone());
    });

    prototype
}

fn define_function_prototype() -> JSObjectRef {
    let prototype = JSObject::new_with_prototype(get_object_prototype());

    prototype.set_key_method("call", |ctx| {
        let this_arg = ctx.arg(0);
    });

    todo!();
}

fn get_object_prototype() -> JSObjectRef {
    OBJECT_PROTOTYPE.with(|value| value.get_or_init(|| define_object_prototype()).clone())
}

fn define_object_instance() -> JSObjectRef {
    let object = JSObject::new();

    object.set_key("prototype", get_object_prototype());

    object
}

pub type JSObjectRef = Rc<JSObject>;

impl JSObject {
    pub fn new_without_prototype() -> JSObjectRef {
        Rc::new(JSObject {
            value: RefCell::new(HashMap::new()),
            prototype: None,
        })
    }

    pub fn new_with_prototype(prototype: JSObjectRef) -> JSObjectRef {
        Rc::new(JSObject {
            value: RefCell::new(HashMap::new()),
            prototype: Some(prototype),
        })
    }

    pub fn new() -> JSObjectRef {
        Rc::new(JSObject {
            value: RefCell::new(HashMap::new()),
            prototype: None,
        })
    }

    pub fn cast(value: &dyn JSValue) -> Option<&JSObject> {
        value.downcast_ref::<JSObject>()
    }

    pub fn cast_rc(value: Rc<dyn JSValue>) -> Option<JSObjectRef> {
        if let Ok(val) = value.as_any_rc().downcast::<JSObject>() {
            Some(val)
        } else {
            None
        }
    }

    pub fn set_key(&self, key: &str, value: JSValueRef) {
        self.value
            .borrow_mut()
            .insert(key.to_string(), JSObjectValue::new_value(value));
    }

    pub fn set_key_method<F>(&self, key: &str, function: F)
    where
        F: Fn(JSFunctionContext) + 'static,
    {
        self.set_key(key, JSFunction::new_named(function, key));
    }

    pub fn get_key(&self, key: &str) -> JSValueRef {
        if let Some(val) = self.value.borrow().get(key) {
            val.value.clone()
        } else {
            JSUndefined::get()
        }
    }
}

impl JSValue for JSObject {
    fn as_any_rc(self: Rc<Self>) -> Rc<dyn Any> {
        self
    }

    fn get_typeof(&self) -> JSStringRef {
        JSString::new("object")
    }

    fn cast_to_boolean(&self) -> JSBooleanRef {
        JSBoolean::get_true()
    }

    fn cast_to_string(&self) -> JSStringRef {
        JSString::new("[object Object]")
    }

    fn cast_to_number(&self) -> JSNumberRef {
        JSNumber::get_nan()
    }

    fn is_equal_to_non_strict(&self, other: &dyn JSValue) -> bool {
        if let Some(other) = JSObject::cast(other) {
            (self as *const JSObject) == (other as *const JSObject)
        } else {
            false
        }
    }

    fn get_debug_string(&self) -> String {
        "[object Object]".to_string()
    }
}
