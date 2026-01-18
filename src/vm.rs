use std::{collections::HashMap, rc::Rc};

use crate::{
    ast::{ASTParser, Expression, FunctionDefinitionExpression, ObjectPropertyName, Statement},
    ecma::{ArrayClass, BooleanClass, FunctionClass, JSModule, ObjectClass, PROTOTYPE},
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
    pub captured_scope: Option<usize>,
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
            captured_scope: None,
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

    pub fn with_captured_scope(mut self, scope_index: usize) -> Object {
        self.captured_scope = Some(scope_index);
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
    Boolean(bool),
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

    pub fn try_as_boolean(&self) -> Option<bool> {
        match self {
            JSValue::Boolean(b) => Some(*b),
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
            JSValue::Boolean(bool) => (if bool { "true" } else { "false" }).to_string(),
        };

        Ok(res)
    }
}

pub struct Scope {
    pub variables: HashMap<String, JSValue>,
}

impl Scope {
    pub fn new() -> Self {
        Scope {
            variables: HashMap::new(),
        }
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
        vm.register_module(ArrayClass::new());
        vm.register_module(BooleanClass::new());

        vm.scopes.push(Scope::new());

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
            Expression::Identifier(identifier) => {
                let value = match identifier.name.as_str() {
                    "true" => JSValue::Boolean(true),
                    "false" => JSValue::Boolean(false),
                    str => self.get_variable(str),
                };

                Ok(value)
            }
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

    // Function tests
    #[test]
    fn test_function_definition() {
        let mut ctx = VM::new();
        let result = ctx
            .evaluate_source(
                r#"
                let f = function() { return 42; };
                f();
            "#,
            )
            .unwrap();
        assert_eq!(result.try_as_number().unwrap(), 42.0);
    }

    #[test]
    fn test_function_with_parameters() {
        let mut ctx = VM::new();
        let result = ctx
            .evaluate_source(
                r#"
                let add = function(a, b) { return a + b; };
                add(5, 3);
            "#,
            )
            .unwrap();
        assert_eq!(result.try_as_number().unwrap(), 8.0);
    }

    #[test]
    fn test_function_with_multiple_parameters() {
        let mut ctx = VM::new();
        let result = ctx
            .evaluate_source(
                r#"
                let calc = function(a, b, c) { return a + b * c; };
                calc(2, 3, 4);
            "#,
            )
            .unwrap();
        assert_eq!(result.try_as_number().unwrap(), 14.0);
    }

    #[test]
    fn test_function_closure() {
        let mut ctx = VM::new();
        let result = ctx
            .evaluate_source(
                r#"
                let x = 10;
                let f = function(y) { return x + y; };
                f(5);
            "#,
            )
            .unwrap();
        assert_eq!(result.try_as_number().unwrap(), 15.0);
    }

    #[test]
    fn test_function_no_parameters() {
        let mut ctx = VM::new();
        let result = ctx
            .evaluate_source(
                r#"
                let getVal = function() { return 100; };
                getVal();
            "#,
            )
            .unwrap();
        assert_eq!(result.try_as_number().unwrap(), 100.0);
    }

    #[test]
    fn test_function_nested_calls() {
        let mut ctx = VM::new();
        let result = ctx
            .evaluate_source(
                r#"
                let double = function(x) { return x * 2; };
                let quad = function(x) { return double(double(x)); };
                quad(5);
            "#,
            )
            .unwrap();
        assert_eq!(result.try_as_number().unwrap(), 20.0);
    }

    // Object tests
    #[test]
    fn test_object_literal_empty() {
        let mut ctx = VM::new();
        let result = ctx.evaluate_source("let obj = {}; obj;").unwrap();
        assert!(result.try_as_object().is_some());
    }

    #[test]
    fn test_object_literal_with_properties() {
        let mut ctx = VM::new();
        ctx.evaluate_source("let obj = { x: 10, y: 20 };").unwrap();
        let result = ctx.evaluate_source("obj.x;").unwrap();
        assert_eq!(result.try_as_number().unwrap(), 10.0);
    }

    #[test]
    fn test_object_property_access() {
        let mut ctx = VM::new();
        let result = ctx
            .evaluate_source(
                r#"
                let person = { age: 25 };
                person.age;
            "#,
            )
            .unwrap();
        assert_eq!(result.try_as_number().unwrap(), 25.0);
    }

    #[test]
    fn test_object_property_assignment() {
        let mut ctx = VM::new();
        let result = ctx
            .evaluate_source(
                r#"
                let obj = { val: 10 };
                obj.val = 20;
                obj.val;
            "#,
            )
            .unwrap();
        assert_eq!(result.try_as_number().unwrap(), 20.0);
    }

    #[test]
    fn test_object_nested_properties() {
        let mut ctx = VM::new();
        let result = ctx
            .evaluate_source(
                r#"
                let obj = { a: 1, b: 2, c: 3 };
                obj.a + obj.b + obj.c;
            "#,
            )
            .unwrap();
        assert_eq!(result.try_as_number().unwrap(), 6.0);
    }

    #[test]
    fn test_object_dynamic_property_assignment() {
        let mut ctx = VM::new();
        let result = ctx
            .evaluate_source(
                r#"
                let obj = {};
                obj.newProp = 42;
                obj.newProp;
            "#,
            )
            .unwrap();
        assert_eq!(result.try_as_number().unwrap(), 42.0);
    }

    // Array tests
    #[test]
    fn test_array_literal_empty() {
        let mut ctx = VM::new();
        let result = ctx.evaluate_source("let arr = []; arr;").unwrap();
        assert!(result.try_as_object().is_some());
    }

    #[test]
    fn test_array_literal_with_elements() {
        let mut ctx = VM::new();
        ctx.evaluate_source("let arr = [1, 2, 3];").unwrap();
        let result = ctx.evaluate_source("arr[0];").unwrap();
        assert_eq!(result.try_as_number().unwrap(), 1.0);
    }

    #[test]
    fn test_array_element_access() {
        let mut ctx = VM::new();
        ctx.evaluate_source("let arr = [10, 20, 30];").unwrap();
        let result = ctx.evaluate_source("arr[1];").unwrap();
        assert_eq!(result.try_as_number().unwrap(), 20.0);
    }

    #[test]
    fn test_array_element_assignment() {
        let mut ctx = VM::new();
        let result = ctx
            .evaluate_source(
                r#"
                let arr = [1, 2, 3];
                arr[1] = 99;
                arr[1];
            "#,
            )
            .unwrap();
        assert_eq!(result.try_as_number().unwrap(), 99.0);
    }

    #[test]
    fn test_array_with_expressions() {
        let mut ctx = VM::new();
        ctx.evaluate_source("let arr = [1 + 1, 2 * 2, 3 + 3];")
            .unwrap();
        let result = ctx.evaluate_source("arr[2];").unwrap();
        assert_eq!(result.try_as_number().unwrap(), 6.0);
    }

    #[test]
    fn test_array_index_with_variable() {
        let mut ctx = VM::new();
        ctx.evaluate_source("let arr = [10, 20, 30];").unwrap();
        ctx.evaluate_source("let i = 2;").unwrap();
        let result = ctx.evaluate_source("arr[i];").unwrap();
        assert_eq!(result.try_as_number().unwrap(), 30.0);
    }

    // Return statement tests
    #[test]
    fn test_return_simple() {
        let mut ctx = VM::new();
        let result = ctx
            .evaluate_source(
                r#"
                let f = function() { return 5; };
                f();
            "#,
            )
            .unwrap();
        assert_eq!(result.try_as_number().unwrap(), 5.0);
    }

    #[test]
    fn test_return_expression() {
        let mut ctx = VM::new();
        let result = ctx
            .evaluate_source(
                r#"
                let f = function(x) { return x * 2; };
                f(7);
            "#,
            )
            .unwrap();
        assert_eq!(result.try_as_number().unwrap(), 14.0);
    }

    #[test]
    fn test_return_early() {
        let mut ctx = VM::new();
        let result = ctx
            .evaluate_source(
                r#"
                let f = function() {
                    return 10;
                    return 20;
                };
                f();
            "#,
            )
            .unwrap();
        assert_eq!(result.try_as_number().unwrap(), 10.0);
    }

    #[test]
    fn test_return_from_nested_block() {
        let mut ctx = VM::new();
        let result = ctx
            .evaluate_source(
                r#"
                let f = function() { { return 42; } };
                f();
            "#,
            )
            .unwrap();
        assert_eq!(result.try_as_number().unwrap(), 42.0);
    }

    #[test]
    fn test_return_with_computation() {
        let mut ctx = VM::new();
        let result = ctx
            .evaluate_source(
                r#"
                let f = function(a, b) { return a * b + 10; };
                f(3, 4);
            "#,
            )
            .unwrap();
        assert_eq!(result.try_as_number().unwrap(), 22.0);
    }

    // Block statement tests
    #[test]
    fn test_block_simple() {
        let mut ctx = VM::new();
        let result = ctx
            .evaluate_source(
                r#"
                let f = function() { return 42; };
                f();
            "#,
            )
            .unwrap();
        assert_eq!(result.try_as_number().unwrap(), 42.0);
    }

    #[test]
    fn test_block_with_variable() {
        let mut ctx = VM::new();
        let result = ctx
            .evaluate_source(
                r#"
                let f = function() {
                    let x = 10;
                    return x;
                };
                f();
            "#,
            )
            .unwrap();
        assert_eq!(result.try_as_number().unwrap(), 10.0);
    }

    #[test]
    fn test_block_multiple_statements() {
        let mut ctx = VM::new();
        let result = ctx
            .evaluate_source(
                r#"
                let f = function() {
                    let a = 5;
                    let b = 3;
                    return a + b;
                };
                f();
            "#,
            )
            .unwrap();
        assert_eq!(result.try_as_number().unwrap(), 8.0);
    }

    #[test]
    fn test_block_nested() {
        let mut ctx = VM::new();
        let result = ctx
            .evaluate_source(
                r#"
                let f = function() {
                    let x = 1;
                    let y = 2;
                    return x + y;
                };
                f();
            "#,
            )
            .unwrap();
        assert_eq!(result.try_as_number().unwrap(), 3.0);
    }

    #[test]
    fn test_block_in_function() {
        let mut ctx = VM::new();
        let result = ctx
            .evaluate_source(
                r#"
                let f = function() {
                    let x = 10;
                    let y = 20;
                    return 30;
                };
                f();
            "#,
            )
            .unwrap();
        assert_eq!(result.try_as_number().unwrap(), 30.0);
    }

    // Combined tests
    #[test]
    fn test_function_returning_object() {
        let mut ctx = VM::new();
        ctx.evaluate_source(
            r#"
            let f = function() { return { val: 42 }; };
        "#,
        )
        .unwrap();
        let result = ctx.evaluate_source("f().val;").unwrap();
        assert_eq!(result.try_as_number().unwrap(), 42.0);
    }

    #[test]
    fn test_function_returning_array() {
        let mut ctx = VM::new();
        ctx.evaluate_source(
            r#"
            let f = function() { return [1, 2, 3]; };
        "#,
        )
        .unwrap();
        ctx.evaluate_source("let result = f();").unwrap();
        let result = ctx.evaluate_source("result[1];").unwrap();
        assert_eq!(result.try_as_number().unwrap(), 2.0);
    }

    #[test]
    fn test_array_of_functions() {
        let mut ctx = VM::new();
        ctx.evaluate_source(
            r#"
            let f1 = function() { return 10; };
            let f2 = function() { return 20; };
            let arr = [f1, f2];
        "#,
        )
        .unwrap();
        ctx.evaluate_source("let fn = arr[0];").unwrap();
        let result = ctx.evaluate_source("fn();").unwrap();
        assert_eq!(result.try_as_number().unwrap(), 10.0);
    }

    #[test]
    fn test_object_with_function_property() {
        let mut ctx = VM::new();
        let result = ctx
            .evaluate_source(
                r#"
                let obj = { method: function(x) { return x * 2; } };
                obj.method(5);
            "#,
            )
            .unwrap();
        assert_eq!(result.try_as_number().unwrap(), 10.0);
    }

    #[test]
    fn test_complex_nested_structure() {
        let mut ctx = VM::new();
        ctx.evaluate_source("let obj = { arr: [1, 2, { inner: 42 }] };")
            .unwrap();
        ctx.evaluate_source("let arrVal = obj.arr;").unwrap();
        ctx.evaluate_source("let innerObj = arrVal[2];").unwrap();
        let result = ctx.evaluate_source("innerObj.inner;").unwrap();
        assert_eq!(result.try_as_number().unwrap(), 42.0);
    }

    #[test]
    fn test_function_with_block_and_return() {
        let mut ctx = VM::new();
        let result = ctx
            .evaluate_source(
                r#"
                let f = function(x) {
                    {
                        let y = x * 2;
                        return y + 5;
                    }
                };
                f(10);
            "#,
            )
            .unwrap();
        assert_eq!(result.try_as_number().unwrap(), 25.0);
    }

    // Nested function tests with returns
    #[test]
    fn test_nested_function_simple_return() {
        let mut ctx = VM::new();
        let result = ctx
            .evaluate_source(
                r#"
                let outer = function() {
                    let inner = function() { return 42; };
                    return inner();
                };
                outer();
            "#,
            )
            .unwrap();
        assert_eq!(result.try_as_number().unwrap(), 42.0);
    }

    #[test]
    fn test_nested_function_return_with_parameter() {
        let mut ctx = VM::new();
        let result = ctx
            .evaluate_source(
                r#"
                let outer = function(x) {
                    let inner = function(y) { return x + y; };
                    return inner(10);
                };
                outer(5);
            "#,
            )
            .unwrap();
        assert_eq!(result.try_as_number().unwrap(), 15.0);
    }

    #[test]
    fn test_nested_function_return_function() {
        let mut ctx = VM::new();
        ctx.evaluate_source(
            r#"
            let makeAdder = function(x) {
                let inner = function(y) {
                    let sum = 5 + 3;
                    return sum;
                };
                return inner;
            };
        "#,
        )
        .unwrap();
        ctx.evaluate_source("let add5 = makeAdder(5);").unwrap();
        let result = ctx.evaluate_source("add5(3);").unwrap();
        assert_eq!(result.try_as_number().unwrap(), 8.0);
    }

    #[test]
    fn test_nested_function_multiple_levels() {
        let mut ctx = VM::new();
        let result = ctx
            .evaluate_source(
                r#"
                let level1 = function(a) {
                    let level2 = function(b) {
                        let level3 = function(c) {
                            return a + b + c;
                        };
                        return level3(3);
                    };
                    return level2(2);
                };
                level1(1);
            "#,
            )
            .unwrap();
        assert_eq!(result.try_as_number().unwrap(), 6.0);
    }

    #[test]
    fn test_nested_function_early_return() {
        let mut ctx = VM::new();
        let result = ctx
            .evaluate_source(
                r#"
                let outer = function(x) {
                    let inner = function() { return x * 2; };
                    return inner();
                    return 999;
                };
                outer(7);
            "#,
            )
            .unwrap();
        assert_eq!(result.try_as_number().unwrap(), 14.0);
    }

    #[test]
    fn test_nested_function_with_computation() {
        let mut ctx = VM::new();
        let result = ctx
            .evaluate_source(
                r#"
                let outer = function(x) {
                    let inner = function(y) { return y * 2; };
                    return inner(x) + 10;
                };
                outer(5);
            "#,
            )
            .unwrap();
        assert_eq!(result.try_as_number().unwrap(), 20.0);
    }

    #[test]
    fn test_nested_function_return_nested_call() {
        let mut ctx = VM::new();
        let result = ctx
            .evaluate_source(
                r#"
                let double = function(x) { return x * 2; };
                let quadruple = function(x) {
                    return double(double(x));
                };
                quadruple(3);
            "#,
            )
            .unwrap();
        assert_eq!(result.try_as_number().unwrap(), 12.0);
    }

    #[test]
    fn test_nested_function_closure_with_return() {
        let mut ctx = VM::new();
        ctx.evaluate_source(
            r#"
            let outer = function(x) {
                let inner = function() { return 50; };
                return inner;
            };
        "#,
        )
        .unwrap();
        ctx.evaluate_source("let fn = outer(5);").unwrap();
        let result = ctx.evaluate_source("fn();").unwrap();
        assert_eq!(result.try_as_number().unwrap(), 50.0);
    }

    #[test]
    fn test_nested_function_multiple_returns() {
        let mut ctx = VM::new();
        let result = ctx
            .evaluate_source(
                r#"
                let outer = function(x) {
                    let inner1 = function() { return x + 1; };
                    let inner2 = function() { return x + 2; };
                    return inner1() + inner2();
                };
                outer(10);
            "#,
            )
            .unwrap();
        assert_eq!(result.try_as_number().unwrap(), 23.0);
    }

    #[test]
    fn test_nested_function_return_with_block() {
        let mut ctx = VM::new();
        let result = ctx
            .evaluate_source(
                r#"
                let outer = function(x) {
                    let inner = function(y) {
                        let z = y + 5;
                        return z * 2;
                    };
                    return inner(x);
                };
                outer(3);
            "#,
            )
            .unwrap();
        assert_eq!(result.try_as_number().unwrap(), 16.0);
    }

    // Boolean tests
    #[test]
    fn test_boolean_literal_true() {
        let mut ctx = VM::new();
        let result = ctx.evaluate_source("true;").unwrap();
        assert_eq!(result.try_as_boolean().unwrap(), true);
    }

    #[test]
    fn test_boolean_literal_false() {
        let mut ctx = VM::new();
        let result = ctx.evaluate_source("false;").unwrap();
        assert_eq!(result.try_as_boolean().unwrap(), false);
    }

    #[test]
    fn test_boolean_constructor_with_truthy_values() {
        let mut ctx = VM::new();
        let result = ctx.evaluate_source("Boolean(1);").unwrap();
        assert_eq!(result.try_as_boolean().unwrap(), true);

        let result = ctx.evaluate_source("Boolean('hello');").unwrap();
        assert_eq!(result.try_as_boolean().unwrap(), true);

        let result = ctx.evaluate_source("Boolean({});").unwrap();
        assert_eq!(result.try_as_boolean().unwrap(), true);
    }

    #[test]
    fn test_boolean_constructor_with_falsy_values() {
        let mut ctx = VM::new();
        let result = ctx.evaluate_source("Boolean(0);").unwrap();
        assert_eq!(result.try_as_boolean().unwrap(), false);

        let result = ctx.evaluate_source("Boolean('');").unwrap();
        assert_eq!(result.try_as_boolean().unwrap(), false);
    }

    #[test]
    fn test_boolean_constructor_with_undefined() {
        let mut ctx = VM::new();
        let result = ctx.evaluate_source("Boolean();").unwrap();
        // Boolean() without arguments should return false, matching JavaScript behavior
        assert_eq!(result.try_as_boolean().unwrap(), false);
    }

    #[test]
    fn test_boolean_in_variable() {
        let mut ctx = VM::new();
        let result = ctx.evaluate_source("let x = true; x;").unwrap();
        assert_eq!(result.try_as_boolean().unwrap(), true);
    }

    #[test]
    fn test_boolean_constructor_with_number() {
        let mut ctx = VM::new();
        let result = ctx.evaluate_source("Boolean(42);").unwrap();
        assert_eq!(result.try_as_boolean().unwrap(), true);

        let result = ctx.evaluate_source("Boolean(-1);").unwrap();
        assert_eq!(result.try_as_boolean().unwrap(), true);
    }
}
