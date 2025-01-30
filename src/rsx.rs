use std::{backtrace::Backtrace, collections::HashMap};

use chumsky::prelude::todo;

use crate::parser::{BinaryOperator, Expression, Program, Statement, UnaryOperator};

#[derive(Debug)]
pub struct RsxError {
    backtrace: Backtrace,
    message: String,
}

impl RsxError {
    pub fn new<T>(message: T, backtrace: Backtrace) -> Self
    where
        T: ToString,
    {
        RsxError {
            message: message.to_string(),
            backtrace,
        }
    }
}

macro_rules! rsx_err {
    ($($arg:tt)*) => {
        RsxError::new(format!("ERROR: {}", format!($($arg)*)), Backtrace::capture())
    };
}

struct Heap {
    address: u64,
    memory: HashMap<u64, Value>,
}

impl Heap {
    pub fn new() -> Self {
        Heap {
            address: 0,
            memory: HashMap::new(),
        }
    }

    pub fn number(&mut self, value: f64) -> HeapRef {
        let address = self.address;
        self.memory.insert(self.address, Value::Number(value));
        self.address += 1;
        HeapRef::new(address)
    }

    pub fn string(&mut self, value: String) -> HeapRef {
        let address = self.address;
        self.memory.insert(self.address, Value::String(value));
        self.address += 1;
        HeapRef::new(address)
    }

    pub fn get(&mut self, address: u64) -> &Value {
        self.memory.get(&address).unwrap()
    }
}

#[derive(Debug)]
pub struct ObjectProperty {
    heap_ref: u64,
}

#[derive(Debug)]
pub enum Value {
    String(String),
    Number(f64),
    Object(HashMap<String, ObjectProperty>),
}

impl Value {
    fn try_number(&self) -> Result<f64, RsxError> {
        match self {
            Value::Number(value) => Ok(*value),
            _ => Err(rsx_err!("Failed to parse {self:#?} to number")),
        }
    }

    fn try_string(&self) -> Result<String, RsxError> {
        match self {
            Value::String(value) => Ok(value.clone()),
            _ => Err(rsx_err!("Failed to parse {self:#?} to number")),
        }
    }
}

struct RsxExecutionContext<'a> {
    parent: Option<&'a mut RsxExecutionContext<'a>>,
    variables: HashMap<String, HeapRef>,
}

impl<'a> RsxExecutionContext<'a> {
    fn new_root() -> RsxExecutionContext<'a> {
        Self {
            parent: None,
            variables: HashMap::new(),
        }
    }

    fn new_child(parent: &'a mut RsxExecutionContext<'a>) -> RsxExecutionContext<'a> {
        Self {
            parent: Some(parent),
            variables: HashMap::new(),
        }
    }
}

#[derive(Clone)]
struct HeapRef {
    address: u64,
}

impl HeapRef {
    fn new(address: u64) -> Self {
        HeapRef { address }
    }

    fn value<'a>(&self, heap: &'a mut Heap) -> &'a Value {
        heap.get(self.address)
    }
}

pub struct Rsx<'a> {
    contexts: Vec<RsxExecutionContext<'a>>,
    stack: Vec<HeapRef>,
    heap: Heap,
}

impl<'a> Rsx<'a> {
    pub fn new() -> Rsx<'a> {
        Rsx {
            heap: Heap::new(),
            contexts: vec![],
            stack: vec![],
        }
    }

    pub fn execute_program(&mut self, program: &'a Program) -> Result<(), RsxError> {
        self.contexts.push(RsxExecutionContext::new_root());

        for statement in &program.statements {
            self.execute_statement(&statement)?;
        }

        self.contexts.pop();

        Ok(())
    }

    fn execute_expression(&mut self, expression: &'a Expression) -> Result<(), RsxError> {
        match expression {
            Expression::Num(value) => self.stack.push(self.heap.number(*value)),
            Expression::String(value) => self.stack.push(self.heap.string(value.clone())),
            Expression::Identifier(name) => {
                let value = self
                    .get_variable(name)
                    .ok_or(rsx_err!("{name} is not defined."))?;

                self.stack.push(value);
            }
            Expression::Unary(expression, operator) => {
                self.execute_expression(expression)?;

                let value = self.pop_stack().value(&mut self.heap).try_number()?;

                let result = match operator {
                    UnaryOperator::NEGATIVE => -value,
                };

                self.stack.push(self.heap.number(result));
            }
            Expression::Binary(left, operator, right) => {
                self.execute_expression(&left)?;

                let left = self.pop_stack().value(&mut self.heap).try_number()?;

                self.execute_expression(&right)?;

                let right = self.pop_stack().value(&mut self.heap).try_number()?;

                let result = match operator {
                    BinaryOperator::ADD => left + right,
                    BinaryOperator::SUB => left - right,
                    BinaryOperator::MULTIPLY => left * right,
                    BinaryOperator::DIV => left / right,
                };

                self.stack.push(self.heap.number(result));
            }
            _ => todo!(),
        };

        Ok(())
    }

    pub fn execute_statement(&mut self, statement: &'a Statement) -> Result<(), RsxError> {
        match statement {
            Statement::Expression(expression) => self.execute_expression(expression)?,
            Statement::Let(name, expression) => {
                self.execute_expression(expression)?;

                let value = self.stack.pop().ok_or(rsx_err!("Failed LET statement"))?;

                if self.current_context().variables.contains_key(name) {
                    return Err(rsx_err!(
                        "Variable {name} already exists in the current scope"
                    ));
                }

                self.current_context()
                    .variables
                    .insert(name.to_string(), value);
            }
            _ => todo!(),
        };
        Ok(())
    }

    fn pop_stack(&mut self) -> HeapRef {
        self.stack.pop().unwrap()
    }

    fn current_context(&mut self) -> &mut RsxExecutionContext<'a> {
        self.contexts.last_mut().unwrap()
    }

    fn get_variable(&mut self, name: &str) -> Option<HeapRef> {
        for context in self.contexts.iter().rev() {
            if let Some(var) = context.variables.get(name) {
                Some(var);
            }
        }

        None
    }

    pub fn last_stack(&mut self) -> &Value {
        self.stack.last().cloned().unwrap().value(&mut self.heap)
    }
}
