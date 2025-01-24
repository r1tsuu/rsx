use std::{any::Any, cell::OnceCell, rc::Rc};

use crate::execution_engine::ExecutionContextRef;

pub enum JSValueType<'a> {
    Number(&'a JSNumber),
    Boolean(&'a JSBoolean),
    String(&'a JSString),
    Function(&'a JSFunction),
    Undefined,
    Null,
}

pub type JSValueRef = Rc<dyn JSValue>;

pub trait JSValue {
    fn as_any(&self) -> &dyn Any;
    fn cast_to_number(&self) -> JSNumberRef;
    fn cast_to_string(&self) -> JSStringRef;
    fn cast_to_boolean(&self) -> JSBooleanRef;
    fn get_debug_name(&self) -> String;
    fn is_equal_to_non_strict(&self, other: &JSValueRef) -> bool;

    fn add(&self, _other: &JSValueRef) -> JSValueRef {
        JSNumber::get_nan()
    }

    fn divide(&self, _other: &JSValueRef) -> JSValueRef {
        JSNumber::get_nan()
    }

    fn substract(&self, _other: &JSValueRef) -> JSValueRef {
        JSNumber::get_nan()
    }

    fn multiply(&self, _other: &JSValueRef) -> JSValueRef {
        JSNumber::get_nan()
    }

    fn retrieve_type(&self) -> JSValueType {
        let as_any = self.as_any();

        if let Some(as_number) = as_any.downcast_ref::<JSNumber>() {
            println!("as number");
            JSValueType::Number(as_number)
        } else if let Some(as_boolean) = as_any.downcast_ref::<JSBoolean>() {
            JSValueType::Boolean(as_boolean)
        } else if let Some(as_string) = as_any.downcast_ref::<JSString>() {
            println!("as str");
            JSValueType::String(as_string)
        } else if let Some(as_function) = as_any.downcast_ref::<JSFunction>() {
            JSValueType::Function(as_function)
        } else if let Some(_) = as_any.downcast_ref::<JSUndefined>() {
            JSValueType::Undefined
        } else if let Some(_) = as_any.downcast_ref::<JSNull>() {
            JSValueType::Null
        } else {
            panic!("Unknown type")
        }
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
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn add(&self, other: &JSValueRef) -> JSValueRef {
        match other.retrieve_type() {
            JSValueType::Number(other) => {
                if let JSNumberValue::Valid(other) = other.value {
                    if let JSNumberValue::Valid(self_value) = self.value {
                        return JSNumber::new(self_value + other);
                    }
                }
            }
            JSValueType::Boolean(other) => {
                if let JSNumberValue::Valid(self_value) = self.value {
                    return JSNumber::new(self_value + other.as_f32());
                }
            }
            JSValueType::String(other) => {
                return JSString::new(&format!("{}{}", self.cast_to_string().value, other.value))
            }
            _ => {}
        }
        JSNumber::get_nan()
    }

    fn substract(&self, other: &JSValueRef) -> JSValueRef {
        if let JSNumberValue::Valid(value) = self.value {
            if let JSValueType::Number(other) = other.retrieve_type() {
                if let JSNumberValue::Valid(other) = other.value {
                    return JSNumber::new(value - other);
                }
            } else if let JSValueType::Boolean(other) = other.retrieve_type() {
                return JSNumber::new(value - other.as_f32());
            }
        }

        return JSNumber::get_nan();
    }

    fn divide(&self, other: &JSValueRef) -> JSValueRef {
        if let JSNumberValue::Valid(value) = self.value {
            if let JSValueType::Number(other) = other.retrieve_type() {
                if let JSNumberValue::Valid(other) = other.value {
                    return JSNumber::new(value / other);
                }
            } else if let JSValueType::Boolean(other) = other.retrieve_type() {
                return JSNumber::new(value / other.as_f32());
            }
        }

        return JSNumber::get_nan();
    }

    fn multiply(&self, other: &JSValueRef) -> JSValueRef {
        if let JSNumberValue::Valid(value) = self.value {
            if let JSValueType::Number(other) = other.retrieve_type() {
                if let JSNumberValue::Valid(other) = other.value {
                    return JSNumber::new(value * other);
                }
            } else if let JSValueType::Boolean(other) = other.retrieve_type() {
                return JSNumber::new(value * other.as_f32());
            }
        }

        return JSNumber::get_nan();
    }

    fn is_equal_to_non_strict(&self, other: &JSValueRef) -> bool {
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

    fn get_debug_name(&self) -> String {
        "Number".to_string()
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
}

impl JSValue for JSString {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn add(&self, other: &JSValueRef) -> JSValueRef {
        JSString::new(&format!("{}{}", self.value, other.cast_to_string().value))
    }

    fn is_equal_to_non_strict(&self, other: &JSValueRef) -> bool {
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

    fn get_debug_name(&self) -> String {
        "String".to_string()
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
}

impl JSValue for JSBoolean {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn add(&self, other: &JSValueRef) -> JSValueRef {
        JSNumber::add(&self.cast_to_number(), other)
    }

    fn divide(&self, other: &JSValueRef) -> JSValueRef {
        JSNumber::divide(&self.cast_to_number(), other)
    }

    fn multiply(&self, other: &JSValueRef) -> JSValueRef {
        JSNumber::multiply(&self.cast_to_number(), other)
    }

    fn substract(&self, other: &JSValueRef) -> JSValueRef {
        JSNumber::substract(&self.cast_to_number(), other)
    }

    fn is_equal_to_non_strict(&self, other: &JSValueRef) -> bool {
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

    fn get_debug_name(&self) -> String {
        "String".to_string()
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
}

impl JSValue for JSUndefined {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn cast_to_boolean(&self) -> JSBooleanRef {
        JSBoolean::get_false()
    }

    fn is_equal_to_non_strict(&self, other: &JSValueRef) -> bool {
        matches!(other.retrieve_type(), JSValueType::Undefined)
    }

    fn cast_to_number(&self) -> JSNumberRef {
        JSNumber::get_nan()
    }

    fn cast_to_string(&self) -> JSStringRef {
        JSString::new("undefined")
    }

    fn get_debug_name(&self) -> String {
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
}

impl JSValue for JSNull {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn cast_to_boolean(&self) -> JSBooleanRef {
        JSBoolean::get_false()
    }

    fn cast_to_number(&self) -> JSNumberRef {
        JSNumber::get_nan()
    }

    fn is_equal_to_non_strict(&self, other: &JSValueRef) -> bool {
        matches!(other.retrieve_type(), JSValueType::Null)
    }

    fn cast_to_string(&self) -> JSStringRef {
        JSString::new("null")
    }

    fn get_debug_name(&self) -> String {
        "null".to_string()
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
}

impl JSValue for JSFunction {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn cast_to_number(&self) -> JSNumberRef {
        JSNumber::get_nan()
    }

    fn is_equal_to_non_strict(&self, other: &JSValueRef) -> bool {
        if let JSValueType::Function(other) = other.retrieve_type() {
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

    fn get_debug_name(&self) -> String {
        "Number".to_string()
    }
}
