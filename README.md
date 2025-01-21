### Javascript Execution Engine in Rust

Very simple JS Engine with basic expressions support:

1. Binary expressions like `1+(3+5)*3*(3+10)`
2. Variables and changing variables:
```js
let x = 10;
let b = 30 + 1;
b = b + 5;
x + b // ExecutionEngine::execute_source evaluates to JavascriptObject { Number { value: 46 }}
```
3. Strings
```js
let x = "Hello World"
x // ExecutionEngine::execute_source evaluates to JavascriptObject { String { value: "Hello World" }}
```

More (functions, objects) soon!