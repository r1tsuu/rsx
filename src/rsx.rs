use std::{
    backtrace::Backtrace,
    cell::{OnceCell, Ref, RefCell},
    collections::HashMap,
    ops::Deref,
    rc::Rc,
    sync::{LazyLock, Mutex, OnceLock},
};

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

pub struct Heap {
    address: u64,
    memory: HashMap<u64, *mut Value>,
}

unsafe impl Send for Heap {}

impl Heap {
    pub fn new() -> Self {
        Heap {
            address: 0,
            memory: HashMap::new(),
        }
    }

    pub fn insert(value: Value) -> HeapRef {
        let mut heap = RSX_HEAP.lock().unwrap();

        let address = heap.address;
        heap.memory.insert(address, Box::into_raw(Box::new(value)));
        heap.address += 1;

        return HeapRef::new(address);
    }

    pub fn alloc_number(value: f64) -> HeapRef {
        Heap::insert(Value::Number(value))
    }

    pub fn alloc_string(value: String) -> HeapRef {
        Heap::insert(Value::String(value))
    }

    pub fn alloc_object() -> HeapRef {
        Heap::insert(Value::Object(Object::new()))
    }

    pub fn get_undefined() -> HeapRef {
        UNDEFINED_CELL
            .get_or_init(|| Heap::insert(Value::Undefined))
            .clone()
    }

    pub fn get_null() -> HeapRef {
        NULL_CELL.get_or_init(|| Heap::insert(Value::Null)).clone()
    }

    pub fn get_false() -> HeapRef {
        FALSE_CELL
            .get_or_init(|| Heap::insert(Value::Boolean(false)))
            .clone()
    }

    pub fn get_true() -> HeapRef {
        TRUE_CELL
            .get_or_init(|| Heap::insert(Value::Boolean(true)))
            .clone()
    }

    pub fn get_boolean(value: bool) -> HeapRef {
        if value {
            Heap::get_true()
        } else {
            Heap::get_false()
        }
    }

    pub fn latest() -> HeapRef {
        let heap = RSX_HEAP.lock().unwrap();
        let address = heap.address - 1;
        HeapRef::new(address)
    }
}

static UNDEFINED_CELL: OnceLock<HeapRef> = OnceLock::new();
static NULL_CELL: OnceLock<HeapRef> = OnceLock::new();
static FALSE_CELL: OnceLock<HeapRef> = OnceLock::new();
static TRUE_CELL: OnceLock<HeapRef> = OnceLock::new();
static RSX_HEAP: LazyLock<Mutex<Heap>> = LazyLock::new(|| Mutex::new(Heap::new()));

#[derive(Debug)]
pub enum Value {
    String(String),
    Number(f64),
    Object(Object),
    Boolean(bool),
    Undefined,
    Null,
}

impl Value {
    pub fn try_number(&self) -> Result<f64, RsxError> {
        match self {
            Value::Number(value) => Ok(*value),
            _ => Err(rsx_err!("Failed to parse {self:#?} to number")),
        }
    }

    pub fn try_string(&self) -> Result<String, RsxError> {
        match self {
            Value::String(value) => Ok(value.clone()),
            _ => Err(rsx_err!("Failed to parse {self:#?} to number")),
        }
    }

    pub fn try_object(&mut self) -> Result<&mut Object, RsxError> {
        match self {
            Value::Object(object) => Ok(object),
            _ => Err(rsx_err!("Failed to parse {self:#?} to object")),
        }
    }
}

#[derive(Debug)]
enum CallableKind {
    FromAST(Vec<String>, Statement),
    Native,
}

#[derive(Debug)]
struct Callable {
    kind: CallableKind,
    captured_context: Rc<RefCell<RsxExecutionContext>>,
}

#[derive(Debug)]
pub struct ObjectProperty {
    heap_ref: HeapRef,
}

impl ObjectProperty {
    pub fn new(heap_ref: HeapRef) -> Self {
        ObjectProperty { heap_ref }
    }
}

#[derive(Debug)]
struct Object {
    properties: HashMap<String, ObjectProperty>,
    callable: Option<Callable>,
}

impl Object {
    fn new() -> Self {
        Self {
            callable: None,
            properties: HashMap::new(),
        }
    }

    fn new_function_from_ast(
        args: Vec<String>,
        statement: Box<Statement>,
        captured_context: Rc<RefCell<RsxExecutionContext>>,
    ) -> Self {
        Self {
            callable: Some(Callable {
                captured_context,
                kind: CallableKind::FromAST(args, *statement),
            }),
            properties: HashMap::new(),
        }
    }

    fn try_callable(&self) -> Result<&Callable, RsxError> {
        match &self.callable {
            Some(callable) => Ok(callable),
            None => Err(rsx_err!("Failed to cast object {self:#?} as callable")),
        }
    }
}

#[derive(Debug)]
struct RsxExecutionContext {
    parent: Option<Rc<RefCell<RsxExecutionContext>>>,
    variables: HashMap<String, HeapRef>,
}

impl RsxExecutionContext {
    fn new_root() -> Rc<RefCell<RsxExecutionContext>> {
        Rc::new(RefCell::new(Self {
            parent: None,
            variables: HashMap::new(),
        }))
    }
}

#[derive(Clone, Debug)]
pub struct HeapRef {
    address: u64,
}

impl HeapRef {
    fn new(address: u64) -> Self {
        HeapRef { address }
    }

    pub fn value<'a>(&self) -> &mut Value {
        let x = *RSX_HEAP.lock().unwrap().memory.get(&self.address).unwrap();
        unsafe { &mut *x }
    }

    pub fn value_raw<'a>(&self) -> *mut Value {
        *RSX_HEAP.lock().unwrap().memory.get(&self.address).unwrap()
    }
}

pub struct Rsx {
    contexts: Vec<Rc<RefCell<RsxExecutionContext>>>,
}

const GLOBAL_THIS: &str = "globalThis";
const GLOBAL_FALSE: &str = "false";
const GLOBAL_TRUE: &str = "true";
const GLOBAL_UNDEFINED: &str = "undefined";
const GLOBAL_NULL: &str = "null";

impl Rsx {
    pub fn new() -> Rsx {
        let mut rsx = Rsx { contexts: vec![] };

        let global_context = Rc::new(RefCell::new(RsxExecutionContext {
            parent: None,
            variables: HashMap::new(),
        }));

        global_context
            .borrow_mut()
            .variables
            .insert(GLOBAL_THIS.to_string(), Heap::alloc_object());

        rsx.contexts.push(global_context);

        rsx.declare_global(GLOBAL_TRUE, Heap::get_true());
        rsx.declare_global(GLOBAL_FALSE, Heap::get_true());
        rsx.declare_global(GLOBAL_UNDEFINED, Heap::get_undefined());
        rsx.declare_global(GLOBAL_NULL, Heap::get_null());

        rsx
    }

    pub fn execute_program(&mut self, program: Program) -> Result<(), RsxError> {
        for statement in &program.statements {
            self.execute_statement(&statement)?;
        }

        Ok(())
    }

    fn get_global_context(&mut self) -> Rc<RefCell<RsxExecutionContext>> {
        self.contexts.get(0).unwrap().clone()
    }

    fn declare_global(&mut self, name: &str, heap_ref: HeapRef) {
        self.get_global_context()
            .borrow_mut()
            .variables
            .get(GLOBAL_THIS)
            // SAFE, created ^
            .unwrap()
            .value()
            .try_object()
            // SAFE, created ^
            .unwrap()
            .properties
            .insert(name.to_string(), ObjectProperty::new(heap_ref));
    }

    fn execute_expression(&mut self, expression: &Expression) -> Result<HeapRef, RsxError> {
        let result = match expression {
            Expression::Num(value) => Heap::alloc_number(*value),
            Expression::String(value) => Heap::alloc_string(value.clone()),
            Expression::Identifier(name) => self
                .get_variable(name)
                .ok_or(rsx_err!("{name} is not defined."))?,
            Expression::Unary(expression, operator) => {
                let value = self.execute_expression(expression)?.value().try_number()?;

                let result = match operator {
                    UnaryOperator::NEGATIVE => -value,
                };

                Heap::alloc_number(result)
            }
            Expression::Binary(left, operator, right) => {
                let left = self.execute_expression(&left)?.value().try_number()?;
                let right = self.execute_expression(&right)?.value().try_number()?;

                let result = match operator {
                    BinaryOperator::ADD => left + right,
                    BinaryOperator::SUB => left - right,
                    BinaryOperator::MULTIPLY => left * right,
                    BinaryOperator::DIV => left / right,
                };

                Heap::alloc_number(result)
            }
            Expression::Call(function, args) => {
                let args = {
                    let mut collected = vec![];
                    for arg in args {
                        collected.push(self.execute_expression(arg)?);
                    }

                    collected
                };

                let function = self.execute_expression(&function)?;
                let callable = function.value().try_object()?.try_callable()?;

                self.spawn_execution_context(callable.captured_context.clone());

                match &callable.kind {
                    CallableKind::FromAST(arg_names, statement) => {
                        for (arg_index, arg_name) in arg_names.iter().enumerate() {
                            self.spawn_execution_context_from_current();
                            if let Some(arg_value) = args.get(arg_index) {
                                self.current_context()
                                    .borrow_mut()
                                    .variables
                                    .insert(arg_name.clone(), arg_value.clone());
                            }

                            self.pop_execution_context();
                        }
                    }
                    _ => unimplemented!(),
                }

                todo!()
            }
            _ => todo!(),
        };

        Ok(result)
    }

    pub fn execute_statement(&mut self, statement: &Statement) -> Result<(), RsxError> {
        match statement {
            Statement::Expression(expression) => {
                self.execute_expression(expression)?;
            }
            Statement::Function(name, args, body) => {
                if self.current_context().borrow().variables.contains_key(name) {
                    return Err(rsx_err!(
                        "Variable {name} already exists in the current scope"
                    ));
                }

                let function = Heap::insert(Value::Object(Object::new_function_from_ast(
                    args.clone(),
                    body.clone(),
                    self.current_context().clone(),
                )));

                self.current_context()
                    .borrow_mut()
                    .variables
                    .insert(name.to_string(), function);
            }
            Statement::Let(name, expression) => {
                self.execute_expression(expression)?;

                let value = self.execute_expression(expression)?;

                if self.current_context().borrow().variables.contains_key(name) {
                    return Err(rsx_err!(
                        "Variable {name} already exists in the current scope"
                    ));
                }

                self.current_context()
                    .borrow_mut()
                    .variables
                    .insert(name.to_string(), value);
            }
            Statement::Block(statements) => {
                self.spawn_execution_context_from_current();

                for statement in statements {
                    self.execute_statement(statement)?;
                }

                self.pop_execution_context();
            }
            _ => todo!(),
        };
        Ok(())
    }

    fn spawn_execution_context(&mut self, execution_context: Rc<RefCell<RsxExecutionContext>>) {
        let ctx = Rc::new(RefCell::new(RsxExecutionContext {
            parent: Some(execution_context),
            variables: HashMap::new(),
        }));

        self.contexts.push(ctx);
    }

    fn spawn_execution_context_from_current(&mut self) {
        let ctx = Rc::new(RefCell::new(RsxExecutionContext {
            parent: Some(self.current_context()),
            variables: HashMap::new(),
        }));

        self.contexts.push(ctx.clone());
    }

    fn pop_execution_context(&mut self) {
        self.contexts.pop();
    }

    fn current_context(&mut self) -> Rc<RefCell<RsxExecutionContext>> {
        self.contexts.last_mut().unwrap().clone()
    }

    fn get_variable(&mut self, name: &str) -> Option<HeapRef> {
        {
            let mut curr_ctx = self.current_context();

            loop {
                if let Some(var) = curr_ctx.borrow().variables.get(name) {
                    return Some(var.clone());
                }

                let parent_context = curr_ctx.borrow().parent.clone();

                if let Some(parent_context) = parent_context {
                    curr_ctx = parent_context.clone();
                } else {
                    break;
                }
            }
        }

        // Try to get from globalThis
        if let Some(var) = self
            .get_global_context()
            .borrow()
            .variables
            .get(GLOBAL_THIS)
            .unwrap()
            .value()
            .try_object()
            .unwrap()
            .properties
            .get(name)
        {
            Some(var.heap_ref.clone())
        } else {
            None
        }
    }
}
