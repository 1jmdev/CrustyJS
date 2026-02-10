# CrustyJS Limitations

This document tracks known deviations and gaps from full ECMAScript behavior.

## Language Coverage

- Parsing supports a practical subset of modern JavaScript, not the full spec grammar.
- AST nodes do not yet carry full source spans for all diagnostics paths.
- `await` support is implemented for async functions in the interpreter, but VM parity is partial.

## Runtime Semantics

- Promise callbacks use deterministic queueing; edge cases around host integration are simplified.
- Error objects and stack traces are lightweight compared to browser/Node stacks.
- Numeric behavior is `f64`-based and may differ in formatting details from major JS engines.

## Modules

- Module loader currently focuses on local file imports.
- Circular imports are detected and reported, but complex live-binding semantics are simplified.
- No package resolution (`node_modules`, package exports maps) is implemented.

## VM

- VM executes a supported opcode subset and bridges unsupported regions back to tree-walk mode.
- Full bytecode parity for all high-level features is still in progress.

## REPL and Tooling

- Completion and highlighting are intentionally lightweight.
- REPL command set is basic (`.help`, `.clear`, `.load`, `.exit`).
