use std::rc::Rc;

use rand::Rng;

use crate::{
    error::EngineError,
    execution_engine::{EngineAddon, ExecutionContext},
    js_value::{JSFunction, JSFunctionContext, JSNumber, JSObject},
};

pub struct MathAddon {}

impl MathAddon {
    pub fn new() -> Rc<Self> {
        Rc::new(MathAddon {})
    }

    fn sqrt(ctx: JSFunctionContext) {
        let a = ctx.arg(0).cast_to_number();
        ctx.set_return(JSNumber::new(a.value.sqrt()));
    }

    fn pow(ctx: JSFunctionContext) {
        let a = ctx.arg(0).cast_to_number();
        let b = ctx.arg(1).cast_to_number();
        ctx.set_return(JSNumber::new(a.value.powf(b.value)));
    }

    fn abs(ctx: JSFunctionContext) {
        let a = ctx.arg(0).cast_to_number();
        ctx.set_return(JSNumber::new(a.value.abs()));
    }

    fn ceil(ctx: JSFunctionContext) {
        let a = ctx.arg(0).cast_to_number();
        ctx.set_return(JSNumber::new(a.value.ceil()));
    }

    fn floor(ctx: JSFunctionContext) {
        let a = ctx.arg(0).cast_to_number();
        ctx.set_return(JSNumber::new(a.value.floor()));
    }

    fn round(ctx: JSFunctionContext) {
        let a = ctx.arg(0).cast_to_number();
        ctx.set_return(JSNumber::new(a.value.round()));
    }

    fn max(ctx: JSFunctionContext) {
        let a = ctx.arg(0).cast_to_number();
        let b = ctx.arg(1).cast_to_number();
        ctx.set_return(JSNumber::new(a.value.max(b.value)));
    }

    fn min(ctx: JSFunctionContext) {
        let a = ctx.arg(0).cast_to_number();
        let b = ctx.arg(1).cast_to_number();
        ctx.set_return(JSNumber::new(a.value.min(b.value)));
    }

    fn random(ctx: JSFunctionContext) {
        let mut rng = rand::thread_rng();
        ctx.set_return(JSNumber::new(rng.gen()));
    }

    fn sign(ctx: JSFunctionContext) {
        let a = ctx.arg(0).cast_to_number();
        ctx.set_return(JSNumber::new(a.value.signum()));
    }

    fn trunc(ctx: JSFunctionContext) {
        let b = ctx.arg(0).cast_to_number();
        ctx.set_return(JSNumber::new(b.value.trunc()));
    }

    fn sin(ctx: JSFunctionContext) {
        let a = ctx.arg(0).cast_to_number();
        ctx.set_return(JSNumber::new(a.value.sin()));
    }

    fn cos(ctx: JSFunctionContext) {
        let a = ctx.arg(0).cast_to_number();
        ctx.set_return(JSNumber::new(a.value.cos()));
    }

    fn tan(ctx: JSFunctionContext) {
        let a = ctx.arg(0).cast_to_number();
        ctx.set_return(JSNumber::new(a.value.tan()));
    }

    fn asin(ctx: JSFunctionContext) {
        let a = ctx.arg(0).cast_to_number();
        ctx.set_return(JSNumber::new(a.value.asin()));
    }

    fn acos(ctx: JSFunctionContext) {
        let a = ctx.arg(0).cast_to_number();
        ctx.set_return(JSNumber::new(a.value.acos()));
    }

    fn atan(ctx: JSFunctionContext) {
        let a = ctx.arg(0).cast_to_number();
        ctx.set_return(JSNumber::new(a.value.atan()));
    }

    fn atan2(ctx: JSFunctionContext) {
        let a = ctx.arg(0).cast_to_number();
        let b = ctx.arg(1).cast_to_number();
        ctx.set_return(JSNumber::new(a.value.atan2(b.value)));
    }

    fn sinh(ctx: JSFunctionContext) {
        let a = ctx.arg(0).cast_to_number();
        ctx.set_return(JSNumber::new(a.value.sinh()));
    }

    fn cosh(ctx: JSFunctionContext) {
        let a = ctx.arg(0).cast_to_number();
        ctx.set_return(JSNumber::new(a.value.cosh()));
    }

    fn tanh(ctx: JSFunctionContext) {
        let a = ctx.arg(0).cast_to_number();
        ctx.set_return(JSNumber::new(a.value.tanh()));
    }

    fn asinh(ctx: JSFunctionContext) {
        let a = ctx.arg(0).cast_to_number();
        ctx.set_return(JSNumber::new(a.value.asinh()));
    }

    fn acosh(ctx: JSFunctionContext) {
        let a = ctx.arg(0).cast_to_number();
        ctx.set_return(JSNumber::new(a.value.acosh()));
    }

    fn atanh(ctx: JSFunctionContext) {
        let a = ctx.arg(0).cast_to_number();
        ctx.set_return(JSNumber::new(a.value.atanh()));
    }

    fn exp(ctx: JSFunctionContext) {
        let a = ctx.arg(0).cast_to_number();
        ctx.set_return(JSNumber::new(a.value.exp()));
    }

    fn expm1(ctx: JSFunctionContext) {
        let a = ctx.arg(0).cast_to_number();
        ctx.set_return(JSNumber::new(a.value.exp_m1()));
    }

    fn log(ctx: JSFunctionContext) {
        let a = ctx.arg(0).cast_to_number();
        ctx.set_return(JSNumber::new(a.value.ln()));
    }

    fn log1p(ctx: JSFunctionContext) {
        let a = ctx.arg(0).cast_to_number();
        ctx.set_return(JSNumber::new(a.value.ln_1p()));
    }

    fn log10(ctx: JSFunctionContext) {
        let a = ctx.arg(0).cast_to_number();
        ctx.set_return(JSNumber::new(a.value.log10()));
    }

    fn log2(ctx: JSFunctionContext) {
        let a = ctx.arg(0).cast_to_number();
        ctx.set_return(JSNumber::new(a.value.log2()));
    }

    fn cbrt(ctx: JSFunctionContext) {
        let a = ctx.arg(0).cast_to_number();
        ctx.set_return(JSNumber::new(a.value.cbrt()));
    }

    fn hypot(ctx: JSFunctionContext) {
        let a = ctx.arg(0).cast_to_number();
        let b = ctx.arg(1).cast_to_number();
        ctx.set_return(JSNumber::new(a.value.hypot(b.value)));
    }

    fn clz32(ctx: JSFunctionContext) {
        let a = ctx.arg(0).cast_to_number();
        ctx.set_return(JSNumber::new(a.value.to_bits().leading_zeros() as f64));
    }
}

impl EngineAddon for MathAddon {
    fn init(&self, ctx: &ExecutionContext) -> Result<(), EngineError> {
        let math = JSObject::new();

        let methods: &[(&str, fn(JSFunctionContext))] = &[
            ("sqrt", MathAddon::sqrt),
            ("pow", MathAddon::pow),
            ("abs", MathAddon::abs),
            ("ceil", MathAddon::ceil),
            ("floor", MathAddon::floor),
            ("round", MathAddon::round),
            ("max", MathAddon::max),
            ("min", MathAddon::min),
            ("random", MathAddon::random),
            ("sign", MathAddon::sign),
            ("trunc", MathAddon::trunc),
            ("sin", MathAddon::sin),
            ("cos", MathAddon::cos),
            ("tan", MathAddon::tan),
            ("asin", MathAddon::asin),
            ("acos", MathAddon::acos),
            ("atan", MathAddon::atan),
            ("atan2", MathAddon::atan2),
            ("sinh", MathAddon::sinh),
            ("cosh", MathAddon::cosh),
            ("tanh", MathAddon::tanh),
            ("asinh", MathAddon::asinh),
            ("acosh", MathAddon::acosh),
            ("atanh", MathAddon::atanh),
            ("exp", MathAddon::exp),
            ("expm1", MathAddon::expm1),
            ("log", MathAddon::log),
            ("log1p", MathAddon::log1p),
            ("log10", MathAddon::log10),
            ("log2", MathAddon::log2),
            ("cbrt", MathAddon::cbrt),
            ("hypot", MathAddon::hypot),
            ("clz32", MathAddon::clz32),
        ];

        for &(name, method) in methods {
            math.set_key_method(name, method);
        }

        ctx.get_global_scope().define("Math", math)?;

        Ok(())
    }
}
