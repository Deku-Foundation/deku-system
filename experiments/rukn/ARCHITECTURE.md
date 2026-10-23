# Rukn implementation architecture

## Current pipeline

```text
UTF-8 source
   │
   ▼
Lexer ── tokens with source locations
   │
   ▼
Parser ── syntax tree
   │
   ▼
Semantic checker ── inferred types, names and scope rules
   │
   ▼
Interpreter ── output or a structured diagnostic
```

The executable and library use the same pipeline. `rukn check` runs lexing, parsing and semantic checks without executing program statements. `rukn run` runs the same checks before interpretation.

## Module boundaries

- `lexer.rs`: tokens, Unicode identifier handling, numeric tokenization, strings, comments and lexical diagnostics.
- `parser.rs`: grammar and operator precedence; it does not execute code.
- `ast.rs`: syntax-tree data structures shared by later passes.
- `typechecker.rs`: declaration scopes, inferred types, operator constraints and assignment validation.
- `interpreter.rs`: values, scopes, evaluation, checked arithmetic and step budgeting.
- `diagnostic.rs`: phase, source location and localized message.
- `main.rs`: CLI argument handling and file I/O only.

Compiler semantics should live in the library, not in the CLI. An editor, formatter, test harness or future language server should reuse the same lexer, parser, semantic checker and diagnostics.

## Target architecture

The interpreter is a vertical-slice prototype, not the final compiler design. The next substantial step is to formalize a typed intermediate representation (IR) and function/module semantics before adding large amounts of syntax. The design should then evaluate:

1. A compact bytecode VM for repeatable execution and straightforward debugging.
2. A native backend (for example, Cranelift or LLVM) against actual language and platform requirements.
3. A stable foreign-function interface and ABI only after data layout, calling conventions, error behavior and ownership rules are specified.

Do not choose a code-generation backend because it is fashionable. Compare startup cost, debug information, cross-compilation, target coverage, binary size, Rust integration, maintenance burden and ability to test generated code.

## Static semantics before systems claims

Stage 0 infers only three types: integer, text and Boolean. It has no borrow checker, lifetimes, algebraic data types, generic constraints, async runtime or memory ownership model. These cannot be inferred from the fact that the interpreter is written in Rust. If Rukn becomes a systems language, ownership/borrowing or an equally explicit memory model must be specified and checked before native memory-safe claims are made.

## Bootstrap direction

Self-hosting is a later verified milestone, not a current capability:

- **Stage 0:** the Rust-hosted reference implementation/compiler with a pinned toolchain.
- **Stage 1:** a Rukn implementation compiled by Stage 0.
- **Stage 2:** compile the Rukn implementation with Stage 1, then test equivalence with Stage 1 output.
- **Stage 3:** repeat builds in clean environments and compare artifacts; investigate nondeterminism rather than hiding differences.

Rust's compiler documents staged bootstrapping, but Rukn should define its own gates. Stage equality is useful evidence; it does not by itself eliminate the general “trusting trust” problem or establish the compiler's correctness.

## Failure policy

Malformed source, unknown tokens, incompatible types, arithmetic overflow, division by zero and step-budget exhaustion return structured diagnostics. The process must not panic on normal invalid user input. Internal invariant failures are bugs and should be covered by tests.
