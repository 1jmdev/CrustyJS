# CrustyJS

CrustyJS is a JavaScript runtime in Rust with two execution engines:

- Tree-walk interpreter (default)
- Bytecode VM (`--vm`)

## Supported Language Features

- Variables, functions, recursion, closures, and arrow functions
- Control flow: `if`/`else`, `while`, `for`, `for...of`
- Arrays and objects with member/index access and assignment
- Prototype lookup and `this` method binding
- Classes with `extends`, `super`, and `instanceof`
- `try/catch/finally`, `throw`, and `new Error(...)`
- Operators: arithmetic, logical, ternary, loose/strict equality, `typeof`

## Usage

Run with tree-walk interpreter:

```sh
cargo run -- examples/fib.js
```

Run with bytecode VM:

```sh
cargo run -- --vm examples/fib.js
```

Inspect tokens/AST/bytecode:

```sh
cargo run -- --tokens examples/fib.js
cargo run -- --ast examples/fib.js
cargo run -- --bytecode examples/fib.js
```

Inline evaluation:

```sh
cargo run -- --eval "console.log(1 + 2)"
```

## VM Architecture

- Lexer/Parser builds AST
- Compiler lowers AST to bytecode `Chunk`
- VM executes opcodes on a value stack with call frames
- For unsupported bytecode regions, VM can bridge to tree-walk execution (`RunTreeWalk`)

## Performance Snapshot

Release benchmark on `examples/fib.js` (`fib(30)` local run):

- `./target/release/crustyjs --vm examples/fib.js` ~ 1.2s
- `./target/release/crustyjs examples/fib.js` ~ 3.7s

The VM path is currently about 3x faster than tree-walk on recursive fib.

## Testing

```sh
cargo test
```
