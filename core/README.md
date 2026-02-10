# CrustyJS

A minimal JavaScript interpreter written in Rust. Supports a subset of JavaScript sufficient to run recursive programs like Fibonacci.

## Supported Features

- Variable declarations (`let`, `const`)
- Functions with recursion
- Control flow (`if`/`else`, `while`)
- Arithmetic and comparison operators
- String literals and concatenation
- `console.log()` output
- Strict equality (`===`, `!==`)
- Unary operators (`-`, `!`)

## Usage

```sh
cargo run -- examples/fib.js
# Output: 55
```

## Example

```js
function fib(n) {
  if (n <= 1) return n;
  return fib(n - 1) + fib(n - 2);
}
console.log(fib(10));
// → 55
```

## Running Tests

```sh
cargo test
```
