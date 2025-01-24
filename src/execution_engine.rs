use std::{cell::RefCell, rc::Rc};

use crate::{
    error::EngineError,
    execution_scope::{ExecutionScope, ExecutionScopeRef},
    js_value::{JSFunctionArgs, JSValue, JSValueKind, JSValueRef},
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
    scopes: RefCell<Vec<ExecutionScopeRef>>,
    call_stack: RefCell<Vec<FunctionCallStackContext>>,
}

impl ExecutionContext {
    fn new() -> Rc<ExecutionContext> {
        Rc::new(ExecutionContext {
            call_stack: RefCell::new(vec![]),
            scopes: RefCell::new(vec![]),
        })
    }

    fn initialize_global_scope(&self) {
        let global_scope = ExecutionScope::new(None);

        global_scope
            .define(UNDEFINED_NAME, JSValue::new_undefined())
            .unwrap();

        global_scope
            .define(TRUE_NAME, JSValue::new_boolean(true))
            .unwrap();

        global_scope
            .define(FALSE_NAME, JSValue::new_boolean(false))
            .unwrap();

        self.scopes.borrow_mut().push(global_scope);
    }

    fn enter_scope(&self) -> ExecutionScopeRef {
        let scope = ExecutionScope::new(Some(self.get_current_scope()));

        self.scopes.borrow_mut().push(scope.clone());

        scope
    }

    fn exit_scope(&self) {
        self.scopes.borrow_mut().pop();
    }

    fn get_global_scope(&self) -> ExecutionScopeRef {
        self.scopes.borrow().get(0).unwrap().clone()
    }

    fn get_undefined(&self) -> JSValueRef {
        self.get_global_scope().get(UNDEFINED_NAME).unwrap()
    }

    fn get_boolean(&self, value: bool) -> JSValueRef {
        self.get_global_scope()
            .get((if value { TRUE_NAME } else { FALSE_NAME }))
            .unwrap()
    }

    fn get_current_scope(&self) -> ExecutionScopeRef {
        self.scopes.borrow().last().unwrap().clone()
    }

    fn set_current_function_error(&self, err: EngineError) -> EngineError {
        self.call_stack.borrow_mut().last_mut().unwrap().error = Some(err.clone());
        err
    }

    fn set_current_function_return(&self, value: JSValueRef) {
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
}

pub type ExecutionContextRef = Rc<ExecutionContext>;

const UNDEFINED_NAME: &str = "undefined";
const TRUE_NAME: &str = "true";
const FALSE_NAME: &str = "false";

impl ExpressionEvaluator {
    fn new() -> Self {
        let ctx = ExecutionContext::new();

        let expression_executor = ExpressionEvaluator { ctx };

        expression_executor.ctx.initialize_global_scope();

        expression_executor
    }

    pub fn evaluate_source<T: ToString>(source: T) -> Result<JSValueRef, EngineError> {
        let mut tokens = vec![];

        for token in Tokenizer::from_source(source.to_string()).to_iter() {
            match token {
                Ok(token) => tokens.push(token),
                Err(err) => return Err(err),
            };
        }

        match Parser::new(tokens).parse_program() {
            Ok(program) => Self::new().evaluate_expression(&program),
            Err(err) => return Err(err),
        }
    }

    fn evaluate_program(&self, expressions: &Vec<Expression>) -> Result<JSValueRef, EngineError> {
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

        return Ok(self.ctx.get_undefined());
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
        Ok(self.ctx.get_undefined())
    }

    fn evaluate_block(&self, expressions: &Vec<Expression>) -> Result<JSValueRef, EngineError> {
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

        Ok(self.ctx.get_undefined())
    }

    fn evaluate_function_declaration(
        &self,
        name: &str,
        parameters: &Vec<Expression>,
        body: &Expression,
    ) -> Result<JSValueRef, EngineError> {
        let parameters = parameters.clone();
        let body = body.clone();

        let func = move |func_ctx: JSFunctionArgs| {
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

        let func = JSValue::new_function(func, Some(name));

        self.ctx.get_current_scope().define(name, func)
    }

    fn evaluate_function_call(
        &self,
        name: &Expression,
        arguments_expressions: &Vec<Expression>,
    ) -> Result<JSValueRef, EngineError> {
        let try_function = self.evaluate_expression(name)?;

        let function = match &try_function.kind {
            JSValueKind::Function { value, .. } => value,
            _ => {
                return Err(EngineError::execution_engine_error(format!(
                    "Tried to call not a function",
                )))
            }
        };

        let mut arguments = vec![];

        for expression in arguments_expressions {
            let value = self.evaluate_expression(expression)?;
            arguments.push(value);
        }

        let context = {
            JSFunctionArgs {
                ctx: self.ctx.clone(),
                js_args: arguments,
            }
        };

        let call = {
            FunctionCallStackContext {
                return_value: self.ctx.get_undefined(),
                function_ptr: try_function.clone(),
                error: None,
                should_return: false,
            }
        };

        self.ctx.call_stack.borrow_mut().push(call);
        self.ctx.enter_scope();
        function(context);
        self.ctx.exit_scope();

        let call = self.ctx.call_stack.borrow_mut().pop().unwrap();

        if let Some(err) = call.error {
            Err(err)
        } else {
            Ok(call.return_value)
        }
    }

    fn evaluate_number_literal(&self, value: f32) -> Result<JSValueRef, EngineError> {
        Ok(JSValue::new_number(value))
    }

    fn evaluate_string_literal(&self, value: &str) -> Result<JSValueRef, EngineError> {
        Ok(JSValue::new_string(value))
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
            TokenKind::EqualsEquals => Ok(self
                .ctx
                .get_boolean(left_result.is_equal_to_non_strict(&right_result))),
            TokenKind::Plus => Ok(JSValue::new_number(
                left_result.cast_to_number() + right_result.cast_to_number(),
            )),
            TokenKind::Minus => Ok(JSValue::new_number(
                left_result.cast_to_number() - right_result.cast_to_number(),
            )),
            TokenKind::Multiply => Ok(JSValue::new_number(
                left_result.cast_to_number() * right_result.cast_to_number(),
            )),
            TokenKind::Divide => Ok(JSValue::new_number(
                left_result.cast_to_number() / right_result.cast_to_number(),
            )),
            _ => Err(EngineError::execution_engine_error(format!(
                "Failed to execute binary expression with operator: {:#?}",
                op
            ))),
        }
    }

    fn evaluate_expression(&self, expression: &Expression) -> Result<JSValueRef, EngineError> {
        let result = match expression {
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
            } => self.evaluate_function_declaration(name, parameters, body),
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
            _ => unimplemented!(),
        };

        result
    }
}
