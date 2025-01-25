use as_any::{AsAny, Downcast};
use std::{cell::OnceCell, rc::Rc};

use crate::execution_engine::ExecutionContextRef;

pub type JSValueRef = Rc<dyn JSValue>;

pub trait JSValue: AsAny {
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

#[derive(Clone)]
pub enum JSNumberValue {
    NaN,
    Valid(f32),
}
pub struct JSNumber {
    pub value: JSNumberValue,
}

thread_local! {
  static NAN: OnceCell<JSNumberRef> = OnceCell::new();
}

pub type JSNumberRef = Rc<JSNumber>;

impl JSNumber {
    pub fn new(value: f32) -> JSNumberRef {
        Rc::new(JSNumber {
            value: JSNumberValue::Valid(value),
        })
    }

    pub fn cast(value: &dyn JSValue) -> Option<&JSNumber> {
        value.downcast_ref::<JSNumber>()
    }

    pub fn get_nan() -> JSNumberRef {
        NAN.with(|value| {
            value
                .get_or_init(|| {
                    return Rc::new(JSNumber {
                        value: JSNumberValue::NaN,
                    });
                })
                .clone()
        })
    }

    pub fn compare(&self, other: JSNumberRef) -> bool {
        if let JSNumberValue::Valid(self_number) = self.value {
            if let JSNumberValue::Valid(other) = other.value {
                return self_number == other;
            }
        }

        return matches!(self.value, JSNumberValue::NaN)
            && matches!(other.value, JSNumberValue::NaN);
    }

    pub fn unwrap_valid_value(&self) -> f32 {
        if let JSNumberValue::Valid(number) = self.value {
            number
        } else {
            panic!("Tried to unwrap NaN")
        }
    }
}

impl JSValue for JSNumber {
    fn add(&self, other: &dyn JSValue) -> JSValueRef {
        if let Some(other) = JSNumber::cast(other) {
            if let JSNumberValue::Valid(other) = other.value {
                if let JSNumberValue::Valid(self_value) = self.value {
                    return JSNumber::new(self_value + other);
                }
            }
        }

        if let Some(other) = JSBoolean::cast(other) {
            if let JSNumberValue::Valid(self_value) = self.value {
                return JSNumber::new(self_value + other.as_f32());
            }
        }

        if let Some(other) = JSString::cast(other) {
            return JSString::new(&format!("{}{}", self.cast_to_string().value, other.value));
        }

        JSNumber::get_nan()
    }

    fn substract(&self, other: &dyn JSValue) -> JSValueRef {
        if let JSNumberValue::Valid(value) = self.value {
            if let Some(other) = JSNumber::cast(other) {
                if let JSNumberValue::Valid(other) = other.value {
                    return JSNumber::new(value - other);
                }
            } else if let Some(other) = JSBoolean::cast(other) {
                return JSNumber::new(value - other.as_f32());
            }
        }

        return JSNumber::get_nan();
    }

    fn divide(&self, other: &dyn JSValue) -> JSValueRef {
        if let JSNumberValue::Valid(value) = self.value {
            if let Some(other) = JSNumber::cast(other) {
                if let JSNumberValue::Valid(other) = other.value {
                    return JSNumber::new(value / other);
                }
            } else if let Some(other) = JSBoolean::cast(other) {
                return JSNumber::new(value / other.as_f32());
            }
        }

        return JSNumber::get_nan();
    }

    fn multiply(&self, other: &dyn JSValue) -> JSValueRef {
        if let JSNumberValue::Valid(value) = self.value {
            if let Some(other) = JSNumber::cast(other) {
                if let JSNumberValue::Valid(other) = other.value {
                    return JSNumber::new(value * other);
                }
            } else if let Some(other) = JSBoolean::cast(other) {
                return JSNumber::new(value * other.as_f32());
            }
        }

        return JSNumber::get_nan();
    }

    fn is_equal_to_non_strict(&self, other: &dyn JSValue) -> bool {
        self.compare(other.cast_to_number())
    }

    fn cast_to_number(&self) -> JSNumberRef {
        match &self.value {
            JSNumberValue::NaN => JSNumber::get_nan(),
            JSNumberValue::Valid(val) => JSNumber::new(*val),
        }
    }

    fn cast_to_string(&self) -> JSStringRef {
        match &self.value {
            JSNumberValue::NaN => JSString::new("NaN"),
            JSNumberValue::Valid(val) => JSString::new(&val.to_string()),
        }
    }

    fn cast_to_boolean(&self) -> JSBooleanRef {
        match &self.value {
            JSNumberValue::NaN => JSBoolean::get_false(),
            JSNumberValue::Valid(val) => {
                if *val == 0.0 {
                    JSBoolean::get_false()
                } else {
                    JSBoolean::get_true()
                }
            }
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
    fn add(&self, other: &dyn JSValue) -> JSValueRef {
        JSString::new(&format!("{}{}", self.value, other.cast_to_string().value))
    }

    fn is_equal_to_non_strict(&self, other: &dyn JSValue) -> bool {
        self.value == other.cast_to_string().value
    }

    fn cast_to_number(&self) -> JSNumberRef {
        self.value
            .parse::<f32>()
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

    pub fn get_false() -> JSBooleanRef {
        FALSE.with(|value| {
            value
                .get_or_init(|| {
                    return Rc::new(JSBoolean { value: false });
                })
                .clone()
        })
    }

    pub fn as_f32(&self) -> f32 {
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
            JSString::new("true")
        } else {
            JSString::new("false")
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
}

impl JSValue for JSUndefined {
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
        JSString::new("undefined")
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
}

impl JSValue for JSNull {
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
        JSString::new("null")
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
