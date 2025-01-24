use std::{cell::RefCell, rc::Rc};

use crate::{
    error::EngineError,
    execution_scope::{ExecutionScope, ExecutionScopeRef},
    js_value::{JSFunctionArgs, JSValue, JSValueRef},
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
    ctx: Rc<RefCell<ExecutionContext>>,
}

pub struct ExecutionContext {
    scopes: Vec<ExecutionScopeRef>,
    execution_tick: u64,
    call_stack: Vec<FunctionCallStackContext>,
}

impl ExecutionContext {
    fn initialize_global_scope(&mut self) {
        let global_scope = Rc::new(RefCell::new(ExecutionScope::new(None)));

        global_scope
            .borrow_mut()
            .define(UNDEFINED_NAME.to_string(), JSValue::new_undefined())
            .unwrap();

        global_scope
            .borrow_mut()
            .define(TRUE_NAME.to_string(), JSValue::new_boolean(true))
            .unwrap();

        global_scope
            .borrow_mut()
            .define(FALSE_NAME.to_string(), JSValue::new_boolean(false))
            .unwrap();

        self.scopes.push(global_scope);
    }

    fn enter_scope(&mut self) -> ExecutionScopeRef {
        let scope = Rc::new(RefCell::new(ExecutionScope::new(Some(
            self.get_current_scope(),
        ))));

        self.scopes.push(scope.clone());

        scope
    }

    fn exit_scope(&mut self) {
        self.scopes.pop();
    }

    fn get_global_scope(&self) -> ExecutionScopeRef {
        self.scopes.get(0).unwrap().clone()
    }

    fn get_undefined(&self) -> JSValueRef {
        self.get_global_scope()
            .borrow()
            .get(UNDEFINED_NAME.to_string())
            .unwrap()
    }

    fn get_boolean(&self, value: bool) -> JSValueRef {
        self.get_global_scope()
            .borrow()
            .get((if value { TRUE_NAME } else { FALSE_NAME }).to_string())
            .unwrap()
    }

    fn get_current_scope(&self) -> ExecutionScopeRef {
        self.scopes.last().unwrap().clone()
    }

    fn get_current_function_call(&mut self) -> &mut FunctionCallStackContext {
        self.call_stack.last_mut().unwrap()
    }

    fn set_current_function_error(&mut self, err: EngineError) -> EngineError {
        self.call_stack.last_mut().unwrap().error = Some(err.clone());
        err
    }

    fn set_current_function_return(&mut self, value: JSValueRef) {
        self.call_stack.last_mut().unwrap().should_return = true;
        self.call_stack.last_mut().unwrap().return_value = value;
    }
}

pub type ExecutionContextRef = Rc<RefCell<ExecutionContext>>;

const UNDEFINED_NAME: &str = "undefined";
const TRUE_NAME: &str = "true";
const FALSE_NAME: &str = "false";

impl ExpressionEvaluator {
    fn new() -> Self {
        let ctx = Rc::new(RefCell::new(ExecutionContext {
            scopes: vec![],
            execution_tick: 0,
            call_stack: vec![],
        }));

        let expression_executor = ExpressionEvaluator { ctx };

        expression_executor
            .ctx
            .borrow_mut()
            .initialize_global_scope();

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
            Ok(program) => Self::new().evaluate_expression(program),
            Err(err) => return Err(err),
        }
    }

    fn evaluate_program(&self, expressions: Vec<Expression>) -> Result<JSValueRef, EngineError> {
        for (index, expr) in expressions.iter().enumerate() {
            match self.evaluate_expression(expr.clone()) {
                Err(err) => return Err(err),
                Ok(value) => {
                    if index == expressions.len() - 1 {
                        return Ok(value);
                    }
                }
            }
        }

        return Ok(self.ctx.borrow().get_undefined());
    }

    fn evaluate_let_variable_declaration(
        &self,
        name: String,
        initializer: Expression,
    ) -> Result<JSValueRef, EngineError> {
        let object = self.evaluate_expression(initializer)?;
        self.ctx
            .borrow_mut()
            .get_current_scope()
            .borrow_mut()
            .define(name, object)
    }

    fn evaluate_function_return(&self, expression: Expression) -> Result<JSValueRef, EngineError> {
        let res = self.evaluate_expression(expression)?;
        self.ctx.borrow_mut().set_current_function_return(res);
        Ok(self.ctx.borrow().get_undefined())
    }

    fn evaluate_block(&self, expressions: Vec<Expression>) -> Result<JSValueRef, EngineError> {
        self.ctx.clone().borrow_mut().enter_scope();

        for (index, expr) in expressions.iter().enumerate() {
            match self.evaluate_expression(expr.clone()) {
                Err(err) => return Err(err),
                Ok(value) => {
                    if self.ctx.borrow().call_stack.len() > 0 {
                        if self.ctx.borrow().call_stack.last().unwrap().should_return {
                            return Ok(self
                                .ctx
                                .borrow()
                                .call_stack
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

        self.ctx.clone().borrow_mut().exit_scope();

        Ok(self.ctx.borrow().get_undefined())
    }

    fn evaluate_function_declaration(
        &self,
        name: String,
        parameters: Vec<Expression>,
        body: Expression,
    ) -> Result<JSValueRef, EngineError> {
        let parameters = parameters.clone();

        let func = move |func_ctx: JSFunctionArgs| {
            for (arg_i, arg) in func_ctx.js_args.iter().enumerate() {
                if let Some(parameter) = parameters.get(arg_i) {
                    match func_ctx
                        .ctx
                        .borrow()
                        .get_current_scope()
                        .borrow_mut()
                        .define(parameter.unwrap_name(), arg.clone())
                    {
                        Err(err) => {
                            func_ctx.ctx.borrow_mut().set_current_function_error(err);

                            return;
                        }
                        _ => {}
                    }
                }
            }

            let executor = ExpressionEvaluator {
                ctx: func_ctx.ctx.clone(),
            };

            let res = executor.evaluate_expression(body.clone());

            if let Err(err) = res {
                executor
                    .ctx
                    .clone()
                    .borrow_mut()
                    .set_current_function_error(err);

                return;
            }
        };

        let func = JSValue::new_function(func);

        self.ctx
            .borrow()
            .get_current_scope()
            .borrow_mut()
            .define(name, func)
    }

    fn evaluate_function_call(
        &self,
        name: Expression,
        arguments_expressions: Vec<Expression>,
    ) -> Result<JSValueRef, EngineError> {
        let try_function = self.evaluate_expression(name)?;

        let function = match try_function.borrow().kind.clone() {
            crate::js_value::JSValueKind::Function { value } => value,
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
                return_value: self.ctx.clone().borrow().get_undefined(),
                function_ptr: try_function.clone(),
                error: None,
                should_return: false,
            }
        };

        self.ctx.borrow_mut().call_stack.push(call);
        self.ctx.borrow_mut().enter_scope();
        function.borrow()(context);
        self.ctx.borrow_mut().exit_scope();

        let call = self.ctx.borrow_mut().call_stack.pop().unwrap();

        if let Some(err) = call.error {
            Err(err)
        } else {
            Ok(call.return_value)
        }
    }

    fn evaluate_number_literal(&self, value: f32) -> Result<JSValueRef, EngineError> {
        Ok(JSValue::new_number(value))
    }

    fn evaluate_string_literal(&self, value: String) -> Result<JSValueRef, EngineError> {
        Ok(JSValue::new_string(value))
    }

    fn evaluate_parenthesized(&self, expression: Expression) -> Result<JSValueRef, EngineError> {
        self.evaluate_expression(expression)
    }

    fn evaluate_identifier(&self, name: String) -> Result<JSValueRef, EngineError> {
        match self
            .ctx
            .borrow()
            .get_current_scope()
            .borrow()
            .get(name.clone())
        {
            Some(value) => Ok(value),
            None => Err(EngineError::execution_engine_error(format!(
                "No variable {} found in the scope",
                name
            ))),
        }
    }

    fn evaluate_binary_op(
        &self,
        left: Expression,
        op: Token,
        right: Expression,
    ) -> Result<JSValueRef, EngineError> {
        if op.is_equals() {
            match left.clone() {
                Expression::Identifier { name } => {
                    match self
                        .ctx
                        .borrow()
                        .get_current_scope()
                        .borrow()
                        .get(name.clone())
                    {
                        Some(var) => var,
                        None => {
                            return Err(EngineError::execution_engine_error(format!(
                                "No variable {} found in the scope",
                                name
                            )))
                        }
                    }
                }
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
                .borrow()
                .get_current_scope()
                .borrow_mut()
                .assign(left.unwrap_name(), value.clone());

            return Ok(value);
        }

        let left_result = self.evaluate_expression(Parser::reorder_expression(left))?;
        let right_result = self.evaluate_expression(Parser::reorder_expression(right))?;

        match op.kind {
            TokenKind::EqualsEquals => Ok(self
                .ctx
                .borrow()
                .get_boolean(left_result.borrow().is_equal_to_non_strict(&right_result))),
            TokenKind::Plus => Ok(JSValue::new_number(
                left_result.borrow().cast_to_number() + right_result.borrow().cast_to_number(),
            )),
            TokenKind::Minus => Ok(JSValue::new_number(
                left_result.borrow().cast_to_number() - right_result.borrow().cast_to_number(),
            )),
            TokenKind::Multiply => Ok(JSValue::new_number(
                left_result.borrow().cast_to_number() * right_result.borrow().cast_to_number(),
            )),
            TokenKind::Divide => Ok(JSValue::new_number(
                left_result.borrow().cast_to_number() / right_result.borrow().cast_to_number(),
            )),
            _ => Err(EngineError::execution_engine_error(format!(
                "Failed to execute binary expression with operator: {:#?}",
                op
            ))),
        }
    }

    fn evaluate_expression(&self, expression: Expression) -> Result<JSValueRef, EngineError> {
        let result = match expression {
            Expression::Program { expressions } => self.evaluate_program(expressions),
            Expression::LetVariableDeclaration { name, initializer } => {
                self.evaluate_let_variable_declaration(name, *initializer)
            }
            Expression::FunctionReturn { expression } => self.evaluate_function_return(*expression),
            Expression::Block { expressions } => self.evaluate_block(expressions),
            Expression::FunctionDeclaration {
                name,
                parameters,
                body,
                ..
            } => self.evaluate_function_declaration(name, parameters, *body),
            Expression::FunctionCall { name, arguments } => {
                self.evaluate_function_call(*name, arguments)
            }
            Expression::NumberLiteral { value } => self.evaluate_number_literal(value),
            Expression::Parenthesized { expression } => self.evaluate_parenthesized(*expression),
            Expression::Identifier { name } => self.evaluate_identifier(name),
            Expression::BinaryOp { left, op, right } => self.evaluate_binary_op(*left, op, *right),
            Expression::StringLiteral { value } => self.evaluate_string_literal(value),
            Expression::FunctionParameter { .. } => Err(EngineError::execution_engine_error(
                "Function parameter cannot be evaluated by its own!",
            )),
            _ => unimplemented!(),
        };

        result
    }
}
