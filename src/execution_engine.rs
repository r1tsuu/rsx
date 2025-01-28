use std::{cell::RefCell, rc::Rc};

use crate::{
    addon_math::MathAddon,
    error::EngineError,
    execution_scope::{ExecutionScope, ExecutionScopeRef},
    js_value::{
        JSBoolean, JSFunction, JSFunctionContext, JSNumber, JSObject, JSObjectRef, JSString,
        JSUndefined, JSValueRef,
    },
    parser::{Expression, Parser},
    tokenizer::{Token, TokenKind, Tokenizer},
};

pub struct FunctionCallStackContext {
    function_ptr: JSValueRef,
    return_value: JSValueRef,
    error: Option<EngineError>,
    should_return: bool,
}

pub struct ExpressionEvaluator {
    ctx: Rc<ExecutionContext>,
}

pub struct ExecutionContext {
    pub scopes: RefCell<Vec<ExecutionScopeRef>>,
    pub call_stack: RefCell<Vec<FunctionCallStackContext>>,
}

pub trait EngineAddon {
    fn init(&self, ctx: &ExecutionContext) -> Result<(), EngineError>;
}

static GLOBAL_THIS: &str = "globalThis";

impl ExecutionContext {
    fn new() -> Result<Rc<ExecutionContext>, EngineError> {
        let ctx = Rc::new(ExecutionContext {
            call_stack: RefCell::new(vec![]),
            scopes: RefCell::new(vec![]),
        });

        let global_scope = ExecutionScope::new(None);
        ctx.scopes.borrow_mut().push(global_scope.clone());

        global_scope.define(GLOBAL_THIS, JSObject::new());

        // Define JS singletons:
        ctx.define_global(JSUndefined::get_name(), JSUndefined::get())?;
        ctx.define_global(JSBoolean::get_true_name(), JSBoolean::get_true())?;
        ctx.define_global(JSBoolean::get_false_name(), JSBoolean::get_false())?;
        ctx.define_global(JSNumber::get_nan_name(), JSNumber::get_nan())?;

        ctx.load_addon(MathAddon::new())?;

        Ok(ctx)
    }

    pub fn get_global_this(&self) -> JSObjectRef {
        JSObject::cast_rc(self.get_global_scope().get(GLOBAL_THIS).unwrap()).unwrap()
    }

    pub fn define_global(&self, name: &str, value: JSValueRef) -> Result<(), EngineError> {
        self.get_global_scope().define(name, value.clone())?;
        JSObject::cast(self.get_global_scope().get(GLOBAL_THIS).unwrap().as_ref())
            .unwrap()
            .set_key(name, value);
        Ok(())
    }

    pub fn enter_scope(&self) -> ExecutionScopeRef {
        let scope = ExecutionScope::new(Some(self.get_current_scope()));

        self.scopes.borrow_mut().push(scope.clone());

        scope
    }

    pub fn exit_scope(&self) {
        self.scopes.borrow_mut().pop();
    }

    pub fn get_global_scope(&self) -> ExecutionScopeRef {
        self.scopes.borrow().get(0).unwrap().clone()
    }

    pub fn get_current_scope(&self) -> ExecutionScopeRef {
        self.scopes.borrow().last().unwrap().clone()
    }

    pub fn set_current_function_error(&self, err: EngineError) -> EngineError {
        self.call_stack.borrow_mut().last_mut().unwrap().error = Some(err.clone());
        err
    }

    pub fn set_current_function_return(&self, value: JSValueRef) {
        self.call_stack
            .borrow_mut()
            .last_mut()
            .unwrap()
            .should_return = true;

        self.call_stack
            .borrow_mut()
            .last_mut()
            .unwrap()
            .return_value = value;
    }

    pub fn load_addon(&self, extension: Rc<dyn EngineAddon>) -> Result<&Self, EngineError> {
        extension.init(self)?;
        Ok(self)
    }
}

pub type ExecutionContextRef = Rc<ExecutionContext>;

impl ExpressionEvaluator {
    pub fn evaluate_source<T: ToString>(source: T) -> Result<JSValueRef, EngineError> {
        let mut tokens = vec![];

        for token in Tokenizer::from_source(source.to_string()).to_iter() {
            match token {
                Ok(token) => tokens.push(token),
                Err(err) => return Err(err),
            };
        }

        match Parser::new(tokens).parse_program() {
            Ok(program) => {
                let evaluator = ExpressionEvaluator {
                    ctx: ExecutionContext::new()?,
                };
                evaluator.evaluate_expression(&program)
            }
            Err(err) => return Err(err),
        }
    }

    fn evaluate_program(&self, expressions: &[Expression]) -> Result<JSValueRef, EngineError> {
        for (index, expr) in expressions.iter().enumerate() {
            match self.evaluate_expression(expr) {
                Err(err) => return Err(err),
                Ok(value) => {
                    if index == expressions.len() - 1 {
                        return Ok(value);
                    }
                }
            }
        }

        return Ok(JSUndefined::get());
    }

    fn evaluate_let_variable_declaration(
        &self,
        name: &str,
        initializer: &Expression,
    ) -> Result<JSValueRef, EngineError> {
        let object = self.evaluate_expression(initializer)?;
        self.ctx.get_current_scope().define(name, object)
    }

    fn evaluate_function_return(&self, expression: &Expression) -> Result<JSValueRef, EngineError> {
        let res = self.evaluate_expression(expression)?;
        self.ctx.set_current_function_return(res);
        Ok(JSUndefined::get())
    }

    fn evaluate_block(&self, expressions: &[Expression]) -> Result<JSValueRef, EngineError> {
        self.ctx.clone().enter_scope();

        for (index, expr) in expressions.iter().enumerate() {
            match self.evaluate_expression(expr) {
                Err(err) => return Err(err),
                Ok(value) => {
                    if self.ctx.call_stack.borrow().len() > 0 {
                        if self.ctx.call_stack.borrow().last().unwrap().should_return {
                            return Ok(self
                                .ctx
                                .call_stack
                                .borrow()
                                .last()
                                .unwrap()
                                .return_value
                                .clone());
                        }
                    }
                    if index == expressions.len() - 1 {
                        return Ok(value);
                    }
                }
            }
        }

        self.ctx.exit_scope();

        Ok(JSUndefined::get())
    }

    fn evaluate_function_declaration(
        &self,
        name: Option<&str>,
        parameters: &[Expression],
        body: &Expression,
    ) -> Result<JSValueRef, EngineError> {
        let parameters = parameters.to_vec().clone();
        let body = body.clone();

        let func = move |func_ctx: JSFunctionContext| {
            for (arg_i, arg) in func_ctx.js_args.iter().enumerate() {
                if let Some(parameter) = parameters.get(arg_i) {
                    match func_ctx
                        .ctx
                        .get_current_scope()
                        .define(&parameter.unwrap_name(), arg.clone())
                    {
                        Err(err) => {
                            func_ctx.ctx.set_current_function_error(err);

                            return;
                        }
                        _ => {}
                    }
                }
            }

            let executor = ExpressionEvaluator {
                ctx: func_ctx.ctx.clone(),
            };

            let res = executor.evaluate_expression(&body);

            if let Err(err) = res {
                executor.ctx.set_current_function_error(err);

                return;
            }
        };

        if let Some(name) = name {
            let func = JSFunction::new(func, Some(name));

            self.ctx.get_current_scope().define(name, func)
        } else {
            Ok(JSFunction::new(func, name))
        }
    }

    fn evaluate_function_call(
        &self,
        name: &Expression,
        arguments_expressions: &[Expression],
    ) -> Result<JSValueRef, EngineError> {
        let try_function = self.evaluate_expression(name)?;

        let function = JSFunction::cast(try_function.as_ref()).ok_or(
            EngineError::execution_engine_error(format!("Tried to call not a function",)),
        )?;

        let mut arguments = vec![];

        for expression in arguments_expressions {
            let value = self.evaluate_expression(expression)?;
            arguments.push(value);
        }

        let context = {
            JSFunctionContext {
                ctx: self.ctx.clone(),
                js_args: arguments,
                this: self.ctx.get_global_this(),
            }
        };

        let call = {
            FunctionCallStackContext {
                return_value: JSUndefined::get(),
                function_ptr: try_function.clone(),
                error: None,
                should_return: false,
            }
        };

        self.ctx.call_stack.borrow_mut().push(call);
        self.ctx.enter_scope();
        (function.value)(context);
        self.ctx.exit_scope();

        let call = self.ctx.call_stack.borrow_mut().pop().unwrap();

        if let Some(err) = call.error {
            Err(err)
        } else {
            Ok(call.return_value)
        }
    }

    fn evaluate_number_literal(&self, value: f64) -> Result<JSValueRef, EngineError> {
        Ok(JSNumber::new(value))
    }

    fn evaluate_string_literal(&self, value: &str) -> Result<JSValueRef, EngineError> {
        Ok(JSString::new(value))
    }

    fn evaluate_parenthesized(&self, expression: &Expression) -> Result<JSValueRef, EngineError> {
        self.evaluate_expression(expression)
    }

    fn evaluate_identifier(&self, name: &str) -> Result<JSValueRef, EngineError> {
        match self.ctx.get_current_scope().get(name) {
            Some(value) => Ok(value),
            None => Err(EngineError::execution_engine_error(format!(
                "No variable {} found in the scope",
                name
            ))),
        }
    }

    fn evaluate_binary_op(
        &self,
        left: &Expression,
        op: &Token,
        right: &Expression,
    ) -> Result<JSValueRef, EngineError> {
        if op.is_equals() {
            match left {
                Expression::Identifier { name } => match self.ctx.get_current_scope().get(name) {
                    Some(var) => var,
                    None => {
                        return Err(EngineError::execution_engine_error(format!(
                            "No variable {} found in the scope",
                            name
                        )))
                    }
                },
                _ => {
                    return Err(EngineError::execution_engine_error(format!(
                        "Expected identifier in assigment, got: {:#?}",
                        left
                    )))
                }
            };

            let value = match self.evaluate_expression(right) {
                Ok(res) => res,
                Err(err) => return Err(err),
            };

            self.ctx
                .get_current_scope()
                .assign(&left.unwrap_name(), value.clone());

            return Ok(value);
        }

        let left_result = self.evaluate_expression(left)?;
        let right_result = self.evaluate_expression(right)?;

        match op.kind {
            TokenKind::EqualsEquals => Ok(JSBoolean::get(
                left_result.is_equal_to_non_strict(right_result.as_ref()),
            )),
            TokenKind::Plus => Ok(left_result.add(right_result.as_ref())),
            TokenKind::Minus => Ok(left_result.substract(right_result.as_ref())),
            TokenKind::Multiply => Ok(left_result.multiply(right_result.as_ref())),
            TokenKind::Divide => Ok(left_result.divide(right_result.as_ref())),
            _ => Err(EngineError::execution_engine_error(format!(
                "Failed to execute binary expression with operator: {:#?}",
                op
            ))),
        }
    }

    fn evaluate_object_literal_expression(
        &self,
        properties: &[Expression],
    ) -> Result<JSValueRef, EngineError> {
        let object = JSObject::new();

        for prop in properties {
            if let Expression::PropertyAssignment { name, initializer } = prop {
                let name = self.evaluate_expression(name)?;
                let initializer = self.evaluate_expression(initializer)?;

                object.set_key(&name.cast_to_string().value, initializer);
            } else {
                return Err(EngineError::execution_engine_error(format!(
                    "Tried to evaluate object literal expression with a property: {prop:#?}"
                )));
            }
        }

        Ok(object)
    }

    fn evaluate_property_access_expression(
        &self,
        name: &Expression,
        expression: &Expression,
    ) -> Result<JSValueRef, EngineError> {
        let obj = self.evaluate_expression(expression)?;
        let obj = JSObject::cast(obj.as_ref()).ok_or(EngineError::execution_engine_error(
            "Tried to access non object property",
        ))?;
        let name = self.evaluate_expression(name)?;
        Ok(obj.get_key(&name.cast_to_string().value))
    }

    fn evaluate_expression(&self, expression: &Expression) -> Result<JSValueRef, EngineError> {
        let result = match &expression {
            Expression::Program { expressions } => self.evaluate_program(expressions),
            Expression::LetVariableDeclaration { name, initializer } => {
                self.evaluate_let_variable_declaration(name, initializer)
            }
            Expression::FunctionReturn { expression } => self.evaluate_function_return(expression),
            Expression::Block { expressions } => self.evaluate_block(expressions),
            Expression::FunctionDeclaration {
                name,
                parameters,
                body,
                ..
            } => self.evaluate_function_declaration(name.as_deref(), parameters, body),
            Expression::FunctionCall { name, arguments } => {
                self.evaluate_function_call(name, arguments)
            }
            Expression::NumberLiteral { value } => self.evaluate_number_literal(*value),
            Expression::Parenthesized { expression } => self.evaluate_parenthesized(expression),
            Expression::Identifier { name } => self.evaluate_identifier(name),
            Expression::BinaryOp { left, op, right } => self.evaluate_binary_op(left, op, right),
            Expression::StringLiteral { value } => self.evaluate_string_literal(value),
            Expression::FunctionParameter { .. } => Err(EngineError::execution_engine_error(
                "Function parameter cannot be evaluated by its own!",
            )),
            Expression::ObjectLiteralExpression { properties } => {
                self.evaluate_object_literal_expression(properties)
            }
            Expression::PropertyAccessExpression { name, expression } => {
                self.evaluate_property_access_expression(name, expression)
            }
            _ => unimplemented!(),
        };

        result
    }
}
