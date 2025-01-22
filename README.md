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
4. Block scoping
```js
// its own scope
{
  let x = 1;
}
```
5. Functions
```js
function x(a) {
  return a + 1;
}

x(1); // Evaluates to 6
```

More  (objects) soon!