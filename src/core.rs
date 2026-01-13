use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    ast::{ASTParser, Expression, Statement},
    error::EngineError,
    lexer::Token,
};

pub type ObjectRef<'exec> = Rc<RefCell<Object<'exec>>>;

pub struct CallContext<'exec> {
    pub executor: &'exec mut ExecutionContext<'exec>,
    pub args: Vec<JSValue<'exec>>,
    pub this: ObjectRef<'exec>,
}

impl<'exec> CallContext<'exec> {
    pub fn arg(&self, index: usize) -> Option<&JSValue<'exec>> {
        self.args.get(index)
    }
}

pub type Call<'exec> = dyn FnMut(CallContext<'exec>) -> JSValue<'exec> + 'exec;
pub type Construct<'exec> = dyn FnMut(CallContext<'exec>) -> JSValue<'exec> + 'exec;

pub struct Object<'exec> {
    pub properties: HashMap<String, JSValue<'exec>>,
    pub prototype: Option<ObjectRef<'exec>>,
    pub call: Option<Box<Call<'exec>>>,
    pub construct: Option<Box<Construct<'exec>>>,
}

impl<'exec> Object<'exec> {
    pub fn new() -> Object<'exec> {
        Object {
            properties: HashMap::new(),
            prototype: None,
            call: None,
            construct: None,
        }
    }

    pub fn build(self) -> ObjectRef<'exec> {
        Rc::new(RefCell::new(self))
    }

    pub fn with_prototype(mut self, prototype: ObjectRef<'exec>) -> Object<'exec> {
        self.prototype = Some(prototype);
        self
    }

    pub fn with_call<F: FnMut(CallContext<'exec>) -> JSValue<'exec> + 'exec>(
        mut self,
        call: F,
    ) -> Object<'exec> {
        self.call = Some(Box::new(call));
        self
    }

    pub fn with_construct<F: FnMut(CallContext<'exec>) -> JSValue<'exec> + 'exec>(
        mut self,
        construct: F,
    ) -> Object<'exec> {
        self.construct = Some(Box::new(construct));
        self
    }

    pub fn with_property(mut self, key: impl Into<String>, value: JSValue<'exec>) -> Self {
        self.properties.insert(key.into(), value);
        self
    }

    pub fn set_property(&mut self, key: impl Into<String>, value: JSValue<'exec>) -> &mut Self {
        self.properties.insert(key.into(), value);
        self
    }

    pub fn get_property(&self, key: &str) -> Option<JSValue<'exec>> {
        self.properties.get(key).cloned()
    }
}

#[derive(Clone)]
pub enum JSValue<'exec> {
    String(String),
    Number(f32),
    Undefined,
    Object(ObjectRef<'exec>),
}

impl<'exec> JSValue<'exec> {
    pub fn try_as_number(&self) -> Option<f32> {
        match self {
            JSValue::Number(n) => Some(*n),
            _ => None,
        }
    }

    pub fn string(str: impl Into<String>) -> JSValue<'exec> {
        JSValue::String(str.into())
    }

    pub fn function<F: FnMut(CallContext<'exec>) -> JSValue<'exec> + 'exec>(
        prototype: ObjectRef<'exec>,
        call: F,
    ) -> JSValue<'exec> {
        JSValue::Object(
            Object::new()
                .with_prototype(prototype.clone())
                .with_call(call)
                .build(),
        )
    }

    pub fn from_object_ref(object_ref: ObjectRef<'exec>) -> JSValue<'exec> {
        JSValue::Object(object_ref.clone())
    }

    pub fn try_as_object(&self) -> Option<ObjectRef<'exec>> {
        match self {
            JSValue::Object(obj) => Some(obj.clone()),
            _ => None,
        }
    }

    pub fn try_as_string(&self) -> Option<&str> {
        match self {
            JSValue::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn add(&self, other: &JSValue) -> JSValue<'exec> {
        if let JSValue::Number(self_number) = self
            && let JSValue::Number(other_number) = other
        {
            return JSValue::Number(*self_number + *other_number);
        }

        unimplemented!()
    }

    pub fn sub(&self, other: &JSValue) -> JSValue<'exec> {
        if let JSValue::Number(self_number) = self
            && let JSValue::Number(other_number) = other
        {
            return JSValue::Number(*self_number - *other_number);
        }

        unimplemented!()
    }

    pub fn multiply(&self, other: &JSValue) -> JSValue<'exec> {
        if let JSValue::Number(self_number) = self
            && let JSValue::Number(other_number) = other
        {
            return JSValue::Number(*self_number * *other_number);
        }

        unimplemented!()
    }

    pub fn divide(&self, other: &JSValue) -> JSValue<'exec> {
        if let JSValue::Number(self_number) = self
            && let JSValue::Number(other_number) = other
        {
            return JSValue::Number(*self_number / *other_number);
        }

        unimplemented!()
    }
}

pub struct Scope<'exec> {
    pub variables: HashMap<String, JSValue<'exec>>,
}

impl<'exec> Scope<'exec> {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: impl Into<String>, value: JSValue<'exec>) {
        self.variables.insert(name.into(), value);
    }
}

pub struct ExecutionContext<'exec> {
    pub scopes: Vec<Scope<'exec>>,
    pub function_prototype: ObjectRef<'exec>,
    pub object_prototype: ObjectRef<'exec>,
    pub global_this: ObjectRef<'exec>,
}

impl<'exec> ExecutionContext<'exec> {
    pub fn new() -> Self {
        let object_prototype = Object::new().build();

        let function_prototype = Object::new()
            .with_prototype(object_prototype.clone())
            .build();

        let construct_object = {
            let object_prototype = object_prototype.clone();
            move |ctx: CallContext<'exec>| {
                ctx.arg(0)
                    .and_then(|v| v.try_as_object())
                    .map(|obj| JSValue::Object(obj.clone()))
                    .unwrap_or_else(|| {
                        JSValue::Object(
                            Object::new()
                                .with_prototype(object_prototype.clone())
                                .build(),
                        )
                    })
            }
        };

        let object_constructor = Object::new()
            .with_prototype(function_prototype.clone())
            .with_construct(construct_object.clone())
            .with_property(
                "prototype",
                JSValue::from_object_ref(object_prototype.clone()),
            )
            .with_call(construct_object)
            .build();

        let function_constructor = Object::new()
            .with_prototype(function_prototype.clone())
            .with_property(
                "prototype",
                JSValue::from_object_ref(function_prototype.clone()),
            )
            .build();

        let js_function_constructor = JSValue::Object(function_constructor);
        let js_object_constructor = JSValue::Object(object_constructor);

        object_prototype
            .borrow_mut()
            .set_property("constructor", js_object_constructor.clone())
            .set_property(
                "toString",
                JSValue::function(function_prototype.clone(), |_| {
                    JSValue::string("[object Object]")
                }),
            );

        function_prototype
            .borrow_mut()
            .set_property("constructor", js_function_constructor.clone());

        let global_this = Object::new()
            .with_prototype(object_prototype.clone())
            .with_property("Object", js_object_constructor)
            .with_property("Function", js_function_constructor)
            .build();

        let ctx = Self {
            scopes: vec![],
            global_this,
            function_prototype,
            object_prototype,
        };

        ctx
    }

    /**
     * Get the value of a variable by searching through the scopes from innermost to outermost.
     * If the variable is not found in any scope, it attempts to retrieve it from the global object.
     * If still not found, it returns JSValue::Undefined.
     */
    fn get_variable(&self, name: &str) -> JSValue<'exec> {
        for scope in self.scopes.iter().rev() {
            if let Some(value) = scope.variables.get(name) {
                return value.clone();
            }
        }

        self.global_this
            .borrow()
            .get_property(name)
            .unwrap_or_else(|| JSValue::Undefined)
    }

    fn get_current_scope_mut(&mut self) -> &mut Scope<'exec> {
        if self.scopes.is_empty() {
            self.scopes.push(Scope::new());
        }

        self.scopes.last_mut().unwrap()
    }

    fn set_variable(&mut self, name: impl Into<String>, value: JSValue<'exec>) {
        self.get_current_scope_mut()
            .variables
            .insert(name.into(), value);
    }

    pub fn execute_expression(&mut self, expression: &Expression) -> JSValue<'exec> {
        match expression {
            Expression::Identifier(identifier) => self.get_variable(&identifier.name),
            Expression::Binary(binary) => {
                let left = self.execute_expression(&binary.left);
                let right = self.execute_expression(&binary.right);

                match binary.operator {
                    Token::Plus => left.add(&right),
                    Token::Minus => left.sub(&right),
                    Token::Star => left.multiply(&right),
                    Token::Slash => left.divide(&right),
                    _ => unimplemented!(),
                }
            }
            Expression::NumericLiteral(numeric) => JSValue::Number(numeric.value),
            _ => unimplemented!(),
        }
    }

    pub fn execute_statement(&mut self, statement: &Statement) -> JSValue<'exec> {
        match statement {
            Statement::Let(let_statement) => {
                let value = self.execute_expression(&let_statement.value);
                self.set_variable(let_statement.name.clone(), value);
                JSValue::Undefined
            }
            Statement::Expression(expression_statement) => {
                self.execute_expression(&expression_statement.expression)
            }
            _ => {
                unimplemented!()
            }
        }
    }

    pub fn evaluate_source(&mut self, source: &str) -> Result<JSValue<'exec>, EngineError> {
        let ast = ASTParser::parse_from_source(source)?;
        Ok(ast
            .iter()
            .map(|statement| self.execute_statement(statement))
            .last()
            .unwrap_or(JSValue::Undefined))
    }
}

#[cfg(test)]
mod tests {
    use crate::core::ExecutionContext;

    #[test]
    fn test_evaluate_numeric_literal() {
        let mut ctx = ExecutionContext::new();
        let result = ctx.evaluate_source("42;").unwrap();
        assert_eq!(result.try_as_number().unwrap(), 42.0);
    }

    #[test]
    fn test_evaluate_addition() {
        let mut ctx = ExecutionContext::new();
        let result = ctx.evaluate_source("5 + 3;").unwrap();
        assert_eq!(result.try_as_number().unwrap(), 8.0);
    }

    #[test]
    fn test_evaluate_subtraction() {
        let mut ctx = ExecutionContext::new();
        let result = ctx.evaluate_source("10 - 4;").unwrap();
        assert_eq!(result.try_as_number().unwrap(), 6.0);
    }

    #[test]
    fn test_evaluate_multiplication() {
        let mut ctx = ExecutionContext::new();
        let result = ctx.evaluate_source("6 * 7;").unwrap();
        assert_eq!(result.try_as_number().unwrap(), 42.0);
    }

    #[test]
    fn test_evaluate_division() {
        let mut ctx = ExecutionContext::new();
        let result = ctx.evaluate_source("20 / 4;").unwrap();
        assert_eq!(result.try_as_number().unwrap(), 5.0);
    }

    #[test]
    fn test_evaluate_complex_expression() {
        let mut ctx = ExecutionContext::new();
        let result = ctx.evaluate_source("2 + 3 * 4;").unwrap();
        assert_eq!(result.try_as_number().unwrap(), 14.0); // 2 + (3 * 4) = 14
    }

    #[test]
    fn test_evaluate_parenthesized_expression() {
        let mut ctx = ExecutionContext::new();
        let result = ctx.evaluate_source("(5 + 3) * 2;").unwrap();
        assert_eq!(result.try_as_number().unwrap(), 16.0); // (5 + 3) * 2 = 16
    }

    #[test]
    fn test_evaluate_let_statement() {
        let mut ctx = ExecutionContext::new();
        let result = ctx.evaluate_source("let x = 42; x;").unwrap();
        assert_eq!(result.try_as_number().unwrap(), 42.0);
    }

    #[test]
    fn test_evaluate_let_with_expression() {
        let mut ctx = ExecutionContext::new();
        let result = ctx.evaluate_source("let y = 10 + 5; y;").unwrap();
        assert_eq!(result.try_as_number().unwrap(), 15.0);
    }

    #[test]
    fn test_evaluate_variable_in_expression() {
        let mut ctx = ExecutionContext::new();
        let result = ctx.evaluate_source("let x = 10; x + 5;").unwrap();
        assert_eq!(result.try_as_number().unwrap(), 15.0);
    }

    #[test]
    fn test_evaluate_multiple_variables() {
        let mut ctx = ExecutionContext::new();
        let result = ctx.evaluate_source("let a = 5; let b = 3; a * b;").unwrap();
        assert_eq!(result.try_as_number().unwrap(), 15.0);
    }

    #[test]
    fn test_evaluate_chained_operations() {
        let mut ctx = ExecutionContext::new();
        let result = ctx.evaluate_source("1 + 2 + 3;").unwrap();
        assert_eq!(result.try_as_number().unwrap(), 6.0);
    }

    #[test]
    fn test_evaluate_variable_reassignment() {
        let mut ctx = ExecutionContext::new();
        let result = ctx.evaluate_source("let x = 10; let x = 20; x;").unwrap();
        assert_eq!(result.try_as_number().unwrap(), 20.0);
    }

    #[test]
    fn test_evaluate_complex_with_variables() {
        let mut ctx = ExecutionContext::new();
        let result = ctx
            .evaluate_source("let a = 2; let b = 3; let c = 4; a + b * c;")
            .unwrap();
        assert_eq!(result.try_as_number().unwrap(), 14.0); // 2 + (3 * 4) = 14
    }
}
