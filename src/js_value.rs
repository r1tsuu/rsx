use as_any::{AsAny, Downcast};

use core::f64;
use std::{any::Any, cell::OnceCell, fmt, rc::Rc};

use crate::execution_engine::ExecutionContextRef;

pub type JSValueRef = Rc<dyn JSValue>;

pub trait JSValue: AsAny {
    fn as_any_rc(self: Rc<Self>) -> Rc<dyn Any>;

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

pub struct JSFunctionArgs {
    pub ctx: ExecutionContextRef,
    pub js_args: Vec<JSValueRef>,
}

pub type JSFunctionValue = Rc<dyn Fn(JSFunctionArgs)>;

pub struct JSFunction {
    pub value: JSFunctionValue,
    pub name: Option<String>,
}

pub type JSFunctionRef = Rc<JSFunction>;

impl JSFunction {
    pub fn new<F>(value: F, name: Option<&str>) -> JSValueRef
    where
        F: Fn(JSFunctionArgs) + 'static,
    {
        Rc::new(JSFunction {
            value: Rc::new(value),
            name: name.map(str::to_string),
        })
    }

    pub fn cast(value: &dyn JSValue) -> Option<&JSFunction> {
        value.downcast_ref::<JSFunction>()
    }
}

impl JSValue for JSFunction {
    fn as_any_rc(self: Rc<Self>) -> Rc<dyn Any> {
        self
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
