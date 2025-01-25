use std::rc::Rc;

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
        let a_as_number = ctx.arg(0).cast_to_number();
        ctx.set_return(JSNumber::new(a_as_number.value.sqrt()));
    }
}

impl EngineAddon for MathAddon {
    fn init(&self, ctx: &ExecutionContext) -> Result<(), EngineError> {
        let scope = ctx.get_global_scope();
        let math = JSObject::new();

        let sqrt = JSFunction::new(MathAddon::sqrt, Some("sqrt"));

        math.set_key("sqrt", &sqrt);
        scope.define("Math", math)?;

        Ok(())
    }
}
