use std::{collections::HashMap, rc::Rc};

use crate::{
    ast::{ASTParser, Expression, FunctionDefinitionExpression, ObjectPropertyName, Statement},
    ecma::{ArrayClass, FunctionClass, JSModule, ObjectClass, PROTOTYPE},
    error::EngineError,
    lexer::Token,
};

#[derive(Clone, Copy, Debug)]
pub struct ObjectRef {
    heap_address: usize,
}

impl ObjectRef {
    pub fn new(heap_address: usize) -> Self {
        Self { heap_address }
    }

    pub fn load(self, vm: &VM) -> &Object {
        vm.heap_get(self)
    }

    pub fn load_mut(self, vm: &mut VM) -> &mut Object {
        vm.heap_get_mut(self)
    }
}

pub struct CallContext {
    pub args: Vec<JSValue>,
    pub this: ObjectRef,
    pub ast_definition: Option<usize>,
}

impl CallContext {
    pub fn new(args: Vec<JSValue>, this: ObjectRef) -> Self {
        Self {
            args,
            this,
            ast_definition: None,
        }
    }

    pub fn new_with_ast(args: Vec<JSValue>, this: ObjectRef, ast_definition: usize) -> Self {
        Self {
            args,
            this,
            ast_definition: Some(ast_definition),
        }
    }

    pub fn arg(&self, index: usize) -> Option<&JSValue> {
        self.args.get(index)
    }
}

pub type NativeFunction = fn(ctx: &mut VM, call_ctx: CallContext) -> Result<JSValue, EngineError>;

#[derive(Clone)]
pub enum Call {
    AST(usize),
    Native(NativeFunction),
}

pub type Construct = NativeFunction;

pub struct Object {
    pub properties: HashMap<String, JSValue>,
    pub prototype: Option<ObjectRef>,
    pub call: Option<Call>,
    pub construct: Option<Construct>,
}

impl Object {
    pub fn new() -> Object {
        Object {
            properties: HashMap::new(),
            prototype: None,
            call: None,
            construct: None,
        }
    }

    pub fn alloc(self, ctx: &mut VM) -> ObjectRef {
        ctx.heap_alloc(self)
    }

    pub fn with_prototype(mut self, prototype: ObjectRef) -> Object {
        self.prototype = Some(prototype);
        self
    }

    pub fn with_call(mut self, call: Call) -> Object {
        self.call = Some(call);
        self
    }

    pub fn with_call_native(mut self, call: NativeFunction) -> Object {
        self.call = Some(Call::Native(call));
        self
    }

    pub fn with_call_ast(mut self, ast_definition: usize) -> Object {
        self.call = Some(Call::AST(ast_definition));
        self
    }

    pub fn with_construct(mut self, construct: Construct) -> Object {
        self.construct = Some(construct);
        self
    }

    pub fn with_property(mut self, key: impl Into<String>, value: JSValue) -> Self {
        self.properties.insert(key.into(), value);
        self
    }

    pub fn set_property(&mut self, key: impl Into<String>, value: JSValue) -> &mut Self {
        self.properties.insert(key.into(), value);
        self
    }

    pub fn delete_property(&mut self, key: &str) -> &mut Self {
        self.properties.remove(key);
        self
    }

    pub fn get_property(&self, key: &str) -> Option<JSValue> {
        self.properties.get(key).cloned()
    }

    pub fn set_prototype(&mut self, prototype: ObjectRef) -> &mut Self {
        self.prototype = Some(prototype);
        self
    }
}

#[derive(Clone, Debug)]
pub enum JSValue {
    String(String),
    Number(f32),
    Undefined,
    Object(ObjectRef),
}

impl JSValue {
    pub fn try_as_number(&self) -> Option<f32> {
        match self {
            JSValue::Number(n) => Some(*n),
            _ => None,
        }
    }

    pub fn string(str: impl Into<String>) -> JSValue {
        JSValue::String(str.into())
    }

    pub fn native_function(prototype: ObjectRef, func: NativeFunction, vm: &mut VM) -> JSValue {
        JSValue::Object(
            Object::new()
                .with_prototype(prototype.clone())
                .with_call_native(func)
                .alloc(vm),
        )
    }

    pub fn from_object_ref(object_ref: ObjectRef) -> JSValue {
        JSValue::Object(object_ref.clone())
    }

    pub fn try_as_object(&self) -> Option<ObjectRef> {
        match self {
            JSValue::Object(obj) => Some(obj.clone()),
            _ => None,
        }
    }

    pub fn try_get_prototype(&self, vm: &VM) -> Option<ObjectRef> {
        match self {
            JSValue::Object(obj) => obj.load(vm).prototype.clone(),
            _ => None,
        }
    }

    pub fn try_as_string(&self) -> Option<String> {
        match self {
            JSValue::String(s) => Some(s.clone()),
            _ => None,
        }
    }

    pub fn add(&self, other: &JSValue) -> JSValue {
        if let JSValue::Number(self_number) = self
            && let JSValue::Number(other_number) = other
        {
            return JSValue::Number(*self_number + *other_number);
        }

        unimplemented!()
    }

    pub fn sub(&self, other: &JSValue) -> JSValue {
        if let JSValue::Number(self_number) = self
            && let JSValue::Number(other_number) = other
        {
            return JSValue::Number(*self_number - *other_number);
        }

        unimplemented!()
    }

    pub fn multiply(&self, other: &JSValue) -> JSValue {
        if let JSValue::Number(self_number) = self
            && let JSValue::Number(other_number) = other
        {
            return JSValue::Number(*self_number * *other_number);
        }

        unimplemented!()
    }

    pub fn divide(&self, other: &JSValue) -> JSValue {
        if let JSValue::Number(self_number) = self
            && let JSValue::Number(other_number) = other
        {
            return JSValue::Number(*self_number / *other_number);
        }

        unimplemented!()
    }

    pub fn cast_to_string(self, vm: &mut VM) -> Result<String, EngineError> {
        let res = match self {
            JSValue::String(s) => s,
            JSValue::Number(n) => n.to_string(),
            JSValue::Object(object) => object
                .load(vm)
                .get_property("toString")
                .and_then(|property| property.try_as_object())
                .map(|object| {
                    vm.call_function(object, object.clone(), vec![])
                        .map(|v| v.try_as_string())
                })
                .unwrap_or_else(|| Ok(Some(ObjectClass::str_fallback())))?
                .unwrap_or_else(ObjectClass::str_fallback),
            JSValue::Undefined => "undefined".to_string(),
        };

        Ok(res)
    }
}

pub struct Scope {
    pub variables: HashMap<String, JSValue>,
}

impl Scope {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: impl Into<String>, value: JSValue) {
        self.variables.insert(name.into(), value);
    }
}

pub struct VM {
    pub scopes: Vec<Scope>,
    pub global_this: ObjectRef,
    pub modules: HashMap<String, Box<dyn JSModule>>,
    pub heap: Vec<Option<Object>>,
    pub heap_free: Vec<usize>,
    pub function_definitions: Vec<Rc<FunctionDefinitionExpression>>,
    pub exit_current_call: bool,
}

impl VM {
    pub fn new() -> Self {
        let global_this = Object::new();
        let mut heap: Vec<Option<Object>> = vec![];
        heap.push(Some(global_this));

        let mut vm = Self {
            function_definitions: vec![],
            scopes: vec![],
            global_this: ObjectRef::new(0),
            modules: HashMap::new(),
            heap,
            heap_free: vec![],
            exit_current_call: false,
        };

        vm.register_module(ObjectClass::new());
        vm.register_module(FunctionClass::new());

        vm
    }

    pub fn heap_alloc(&mut self, object: Object) -> ObjectRef {
        if let Some(free_address) = self.heap_free.pop() {
            self.heap[free_address] = Some(object);
            return ObjectRef::new(free_address);
        }

        self.heap.push(Some(object));
        ObjectRef::new(self.heap.len() - 1)
    }

    pub fn heap_get(&self, object_ref: ObjectRef) -> &Object {
        self.heap
            .get(object_ref.heap_address)
            .expect(
                format!(
                    "Invalid heap address: {}. This is likely a bug in GC handling",
                    object_ref.heap_address
                )
                .as_str(),
            )
            .as_ref()
            .expect("Object at heap address is None. This is likely a bug in GC handling")
    }

    pub fn heap_get_mut(&mut self, object_ref: ObjectRef) -> &mut Object {
        self.heap
            .get_mut(object_ref.heap_address)
            .expect(
                format!(
                    "Invalid heap address: {}. This is likely a bug in GC handling",
                    object_ref.heap_address
                )
                .as_str(),
            )
            .as_mut()
            .expect("Object at heap address is None. This is likely a bug in GC handling")
    }

    pub fn heap_free(&mut self, object_ref: ObjectRef) {
        self.heap[object_ref.heap_address] = None;
        self.heap_free.push(object_ref.heap_address);
    }

    fn register_module(&mut self, module: impl JSModule + 'static) {
        let mut module_instance = module;

        module_instance.init(self);

        self.modules.insert(
            module_instance.name().to_string(),
            Box::new(module_instance),
        );
    }

    pub fn global_constructor_prototype(&self, name: &str) -> Option<ObjectRef> {
        self.global_this
            .load(self)
            .get_property(name)
            .and_then(|value| value.try_as_object())
            .and_then(|object| object.load(self).get_property(PROTOTYPE))
            .and_then(|value| value.try_as_object())
    }

    /**
     * Get the value of a variable by searching through the scopes from innermost to outermost.
     * If the variable is not found in any scope, it attempts to retrieve it from the global object.
     * If still not found, it returns JSValue::Undefined.
     */
    fn get_variable(&self, name: &str) -> JSValue {
        for scope in self.scopes.iter().rev() {
            if let Some(value) = scope.variables.get(name) {
                return value.clone();
            }
        }

        self.global_this
            .load(self)
            .get_property(name)
            .unwrap_or_else(|| JSValue::Undefined)
    }

    fn get_current_scope_mut(&mut self) -> &mut Scope {
        if self.scopes.is_empty() {
            self.scopes.push(Scope::new());
        }

        self.scopes.last_mut().unwrap()
    }

    pub fn get_variable_from_global(&self, name: &str) -> Option<JSValue> {
        self.global_this.load(self).get_property(name)
    }

    fn assign_variable(&mut self, name: &str, value: JSValue) -> Result<(), EngineError> {
        for scope in self.scopes.iter_mut().rev() {
            if scope.variables.contains_key(name) {
                scope.variables.insert(name.to_string(), value);
                return Ok(());
            }
        }

        self.global_this
            .load_mut(self)
            .get_property(name)
            .map(|_| {
                self.global_this.load_mut(self).set_property(name, value);
                ()
            })
            .ok_or_else(|| {
                EngineError::js(format!("Tried to assign to undefined variable '{}'", name))
            })
    }

    pub fn set_variable(&mut self, name: impl Into<String>, value: JSValue) {
        self.get_current_scope_mut()
            .variables
            .insert(name.into(), value);
    }

    pub fn call_function(
        &mut self,
        function: ObjectRef,
        this: ObjectRef,
        args: Vec<JSValue>,
    ) -> Result<JSValue, EngineError> {
        let function_object = function.load(self);

        let call = function_object
            .call
            .as_ref()
            .ok_or_else(|| EngineError::js("Tried to call a non-callable object"))?;

        let call_ctx = CallContext::new(args, this);

        match call {
            Call::Native(native_function) => native_function(self, call_ctx),
            Call::AST(ast) => {
                let definition = self
                    .function_definitions
                    .get(*ast)
                    .ok_or_else(|| {
                        EngineError::js(format!("No AST definition with index={} found", *ast))
                    })?
                    .clone();

                self.scopes.push(Scope::new());

                for (arg_index, arg_name) in definition.arguments.iter().enumerate() {
                    let arg_value = call_ctx
                        .arg(arg_index)
                        .cloned()
                        .unwrap_or(JSValue::Undefined);
                    self.set_variable(arg_name, arg_value);
                }

                let res = self.execute_statement(&Statement::block(definition.block.body.clone()));

                self.scopes.pop();

                res
            }
        }
    }

    pub fn execute_expression(&mut self, expression: &Expression) -> Result<JSValue, EngineError> {
        match expression {
            Expression::Identifier(identifier) => Ok(self.get_variable(&identifier.name)),
            Expression::Binary(binary) => {
                if matches!(binary.operator, Token::Equal) {
                    let right = self.execute_expression(&binary.right)?;

                    if let Some(identifier) = binary.left.try_as_identifier() {
                        self.assign_variable(&identifier.name, right.clone())?;
                        return Ok(right);
                    }

                    if let Some(property_access) = binary.left.try_as_property_access() {
                        self.execute_expression(&property_access.expression)?
                            .try_as_object()
                            .ok_or_else(|| {
                                EngineError::js(format!(
                                    "Tried to access property of non-object: {:#?}",
                                    property_access.expression
                                ))
                            })?
                            .load_mut(self)
                            .set_property(&property_access.property, right.clone());

                        return Ok(right);
                    }

                    if let Some(element_access) = binary.left.try_as_element_access() {
                        let object = self
                            .execute_expression(&element_access.expression)?
                            .try_as_object()
                            .ok_or_else(|| {
                                EngineError::js(format!(
                                    "Tried to access element of non-object: {:#?}",
                                    element_access.expression
                                ))
                            })?;

                        let key = self.execute_expression(&element_access.element)?;
                        let key_string = key.cast_to_string(self)?;

                        object
                            .load_mut(self)
                            .set_property(key_string, right.clone());

                        return Ok(right);
                    }

                    return Err(EngineError::js(format!(
                        "Invalid left-hand side in assignment: {:#?}",
                        binary.left
                    )));
                }

                let left = self.execute_expression(&binary.left)?;
                let right = self.execute_expression(&binary.right)?;

                match binary.operator {
                    Token::Plus => Ok(left.add(&right)),
                    Token::Minus => Ok(left.sub(&right)),
                    Token::Star => Ok(left.multiply(&right)),
                    Token::Slash => Ok(left.divide(&right)),
                    _ => unimplemented!(),
                }
            }
            Expression::NumericLiteral(numeric) => Ok(JSValue::Number(numeric.value)),
            Expression::ObjectLiteral(object_literal) => {
                let mut object = ObjectClass::create(self);

                for prop in object_literal.properties.iter() {
                    let name = match &prop.name {
                        ObjectPropertyName::Name(string) => string,
                        ObjectPropertyName::Computed(expression) => {
                            &self.execute_expression(expression)?.cast_to_string(self)?
                        }
                    };

                    object.set_property(name, self.execute_expression(&prop.value)?);
                }

                Ok(JSValue::Object(object.alloc(self)))
            }
            Expression::ArrayLiteral(array_literal) => {
                let array = ArrayClass::create(self).alloc(self);

                for element in &array_literal.elements {
                    let value = self.execute_expression(&element)?;
                    ArrayClass::push(self, CallContext::new(vec![value], array.clone()))?;
                }

                Ok(JSValue::Object(array))
            }
            Expression::PropertyAccess(property_access) => {
                let value = self
                    .execute_expression(&property_access.expression)?
                    .try_as_object()
                    .ok_or_else(|| {
                        EngineError::js(format!(
                            "Tried to access property of non-object: {:#?}",
                            property_access.expression
                        ))
                    })?
                    .load(self)
                    .get_property(&property_access.property)
                    .unwrap_or(JSValue::Undefined);

                Ok(value)
            }
            Expression::ElementAccess(element_access) => {
                let object = self
                    .execute_expression(&element_access.expression)?
                    .try_as_object()
                    .ok_or_else(|| {
                        EngineError::js(format!(
                            "Tried to access element of non-object: {:#?}",
                            element_access.expression
                        ))
                    })?;

                let key = self.execute_expression(&element_access.element)?;
                let key_string = key.cast_to_string(self)?;

                Ok(object
                    .load(self)
                    .get_property(&key_string)
                    .unwrap_or(JSValue::Undefined))
            }
            Expression::FunctionCall(function_call) => {
                let function_object = self
                    .execute_expression(&function_call.function)?
                    .try_as_object()
                    .ok_or_else(|| {
                        EngineError::js(format!(
                            "Tried to call non-function: {:#?}",
                            function_call.function
                        ))
                    })?;

                let mut args: Vec<JSValue> = vec![];

                for expr in &function_call.arguments {
                    args.push(self.execute_expression(&expr)?);
                }

                self.exit_current_call = false;
                self.call_function(function_object, self.global_this, args)
            }
            Expression::FunctionDefinition(function_definition) => Ok(JSValue::Object(
                FunctionClass::create_from_ast(self, function_definition.clone()).alloc(self),
            )),
        }
    }

    pub fn execute_statement(&mut self, statement: &Statement) -> Result<JSValue, EngineError> {
        match statement {
            Statement::Let(let_statement) => {
                let value = self.execute_expression(&let_statement.value)?;
                self.set_variable(let_statement.name.clone(), value);
                Ok(JSValue::Undefined)
            }
            Statement::Expression(expression_statement) => {
                self.execute_expression(&expression_statement.expression)
            }
            Statement::Return(return_statement) => {
                let return_value = self.execute_expression(&return_statement.expression);
                self.exit_current_call = true;

                return_value
            }
            Statement::Block(block_statement) => {
                for statement in &block_statement.body {
                    let value = self.execute_statement(statement)?;

                    if self.exit_current_call {
                        return Ok(value);
                    }
                }

                Ok(JSValue::Undefined)
            }
            Statement::If(_if_statement) => {
                unimplemented!()
            }
        }
    }

    pub fn evaluate_source(&mut self, source: &str) -> Result<JSValue, EngineError> {
        let ast = ASTParser::parse_from_source(source)?;

        ast.iter()
            .map(|statement| self.execute_statement(statement))
            .last()
            .unwrap_or(Ok(JSValue::Undefined))
    }
}

#[cfg(test)]
mod tests {
    use crate::vm::VM;

    #[test]
    fn test_evaluate_numeric_literal() {
        let mut ctx = VM::new();
        let result = ctx.evaluate_source("42;").unwrap();
        assert_eq!(result.try_as_number().unwrap(), 42.0);
    }

    #[test]
    fn test_evaluate_addition() {
        let mut ctx = VM::new();
        let result = ctx.evaluate_source("5 + 3;").unwrap();
        assert_eq!(result.try_as_number().unwrap(), 8.0);
    }

    #[test]
    fn test_evaluate_subtraction() {
        let mut ctx = VM::new();
        let result = ctx.evaluate_source("10 - 4;").unwrap();
        assert_eq!(result.try_as_number().unwrap(), 6.0);
    }

    #[test]
    fn test_evaluate_multiplication() {
        let mut ctx = VM::new();
        let result = ctx.evaluate_source("6 * 7;").unwrap();
        assert_eq!(result.try_as_number().unwrap(), 42.0);
    }

    #[test]
    fn test_evaluate_division() {
        let mut ctx = VM::new();
        let result = ctx.evaluate_source("20 / 4;").unwrap();
        assert_eq!(result.try_as_number().unwrap(), 5.0);
    }

    #[test]
    fn test_evaluate_complex_expression() {
        let mut ctx = VM::new();
        let result = ctx.evaluate_source("2 + 3 * 4;").unwrap();
        assert_eq!(result.try_as_number().unwrap(), 14.0); // 2 + (3 * 4) = 14
    }

    #[test]
    fn test_evaluate_parenthesized_expression() {
        let mut ctx = VM::new();
        let result = ctx.evaluate_source("(5 + 3) * 2;").unwrap();
        assert_eq!(result.try_as_number().unwrap(), 16.0); // (5 + 3) * 2 = 16
    }

    #[test]
    fn test_evaluate_let_statement() {
        let mut ctx = VM::new();
        let result = ctx.evaluate_source("let x = 42; x;").unwrap();
        assert_eq!(result.try_as_number().unwrap(), 42.0);
    }

    #[test]
    fn test_evaluate_let_with_expression() {
        let mut ctx = VM::new();
        let result = ctx.evaluate_source("let y = 10 + 5; y;").unwrap();
        assert_eq!(result.try_as_number().unwrap(), 15.0);
    }

    #[test]
    fn test_evaluate_variable_in_expression() {
        let mut ctx = VM::new();
        let result = ctx.evaluate_source("let x = 10; x + 5;").unwrap();
        assert_eq!(result.try_as_number().unwrap(), 15.0);
    }

    #[test]
    fn test_evaluate_multiple_variables() {
        let mut ctx = VM::new();
        let result = ctx.evaluate_source("let a = 5; let b = 3; a * b;").unwrap();
        assert_eq!(result.try_as_number().unwrap(), 15.0);
    }

    #[test]
    fn test_evaluate_chained_operations() {
        let mut ctx = VM::new();
        let result = ctx.evaluate_source("1 + 2 + 3;").unwrap();
        assert_eq!(result.try_as_number().unwrap(), 6.0);
    }

    #[test]
    fn test_evaluate_variable_reassignment() {
        let mut ctx = VM::new();
        let result = ctx.evaluate_source("let x = 10; let x = 20; x;").unwrap();
        assert_eq!(result.try_as_number().unwrap(), 20.0);
    }

    #[test]
    fn test_evaluate_complex_with_variables() {
        let mut ctx = VM::new();
        let result = ctx
            .evaluate_source("let a = 2; let b = 3; let c = 4; a + b * c;")
            .unwrap();
        assert_eq!(result.try_as_number().unwrap(), 14.0); // 2 + (3 * 4) = 14
    }
}
