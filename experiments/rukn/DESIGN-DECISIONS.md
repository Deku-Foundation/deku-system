# Rukn design-decision register

This document tracks decisions that must be made before the first public 0.1 release. It separates current Stage-0 behavior from proposed long-term semantics. A prototype implementation is not automatically a language design decision.

## Project objective

Rukn (رُكن) is intended to become an Arabic-first, statically checked compiled programming language with a path to self-hosting. Stage 0 is a small Rust-hosted interpreter. The goal is not to mechanically translate Rust keywords into Arabic; it is to create a coherent language whose grammar, diagnostics, type rules, tooling and ecosystem work well with Arabic source code and remain practical for system-level programming.

## Decision table

| ID | Topic | Current Stage-0 behavior | Proposed direction | Status |
|---|---|---|---|---|
| RKN-001 | Language surface | Arabic keywords, conventional symbolic operators, Arabic or Latin identifiers | Keep Arabic as the canonical keyword surface. Do not parse free-form natural language. Consider an explicit optional English-keyword profile only if real contributors need it; avoid two implicit grammars. | Proposal |
| RKN-002 | Unicode identifiers | Unicode XID, underscore, NFC normalization, case-sensitive names | Keep NFC and make all identifier-equivalence rules normative. Do not strip diacritics or equate different Arabic letters. Add warnings for mixed scripts/confusable names using UTS #39 research. | Stage-0 implemented; warning policy deferred |
| RKN-003 | Directional/invisible characters | Selected controls are rejected; digit scripts cannot be mixed within one numeral | Keep controls forbidden in identifiers, comments and literals unless a use case is explicitly specified. Add test fixtures for visual spoofing and editor rendering. | Stage-0 implemented; broader spoof detection deferred |
| RKN-004 | Integer model | Signed 64-bit integers with checked operations; minimum signed integer cannot be spelled directly yet | Resolve numeric-literal range before 0.1; specify overflow, division/remainder and literal typing. Evaluate arbitrary precision only as a deliberate language/library choice. | Open |
| RKN-005 | Binding mutability | `دع` declares mutable bindings; `ثابت` declares immutable bindings | Decide whether immutability should be the default before adding functions and public APIs. Do not silently change this after code examples spread. | Open |
| RKN-006 | Type system | Inferred local integer, text and Boolean types; assignments preserve type | Remain statically typed. Define explicit type annotations, function signatures, unit/never types, and a documented coercion policy before adding more syntax. No implicit truthiness or silent numeric narrowing. | Proposal |
| RKN-007 | Error handling | Errors are diagnostics; no user-defined result/option types yet | Design explicit error/result types and control-flow semantics before filesystem/network APIs. Avoid implicit exceptions as the hidden default for systems code. | Deferred |
| RKN-008 | Memory safety | The prototype interpreter runs on Rust; Rukn code has no ownership model | Do not infer memory safety from the host implementation. If targeting native systems code, formally choose ownership/borrowing, a managed runtime, or another memory model and prove what guarantees each mode offers. | Open, release blocker for safety claims |
| RKN-009 | Compiler architecture | Lexer → parser/AST → semantic checker → interpreter | Keep one semantic front-end API shared by CLI, formatter, language server and all execution backends. Introduce a typed IR before a bytecode VM or native backend. | Stage-0 accepted |
| RKN-010 | Native backend | No native code generation | Benchmark Cranelift and LLVM against the same Rukn corpus only after type and IR semantics stabilize. Compare correctness, debug information, cross-compilation, binary size, startup, target coverage and maintenance—not novelty. | Deferred |
| RKN-011 | Foreign-function interface | No FFI | Specify data layout, calling conventions, ownership transfer, error propagation and ABI versioning before FFI is implemented. Never allow an undocumented escape hatch from safety rules. | Deferred |
| RKN-012 | Standard library | No standard library beyond printing in the interpreter | Start with a small, portable core. Separate deterministic pure APIs from I/O capabilities. Version the library contract alongside the language, with compatibility tests. | Deferred |
| RKN-013 | Package and module model | No modules, imports or package manager | Define module identity, source resolution, visibility, cyclic dependencies, lockfiles, compilation units and supply-chain trust before the language package manager. Keep packages separate from arbitrary build scripts. | Deferred |
| RKN-014 | Self-hosting | Not implemented | Maintain explicit Stage-0/Stage-1/Stage-2 bootstrap procedures. A self-hosting claim requires the Rukn compiler to compile its own implementation and a clean rebuild comparison. | Deferred |
| RKN-015 | Stable compatibility | Stage-0 changes can be breaking | Version the language specification, compiler behavior, standard library, package format and ABI independently. Freeze compatibility promises only after conformance tests and migration policy exist. | Proposal |

## Required design experiments before syntax expansion

### 1. Binding and type experiment

Create a small table of valid/invalid examples for mutable/immutable declarations, branch scopes, shadowing, type inference, annotations, function parameters and return values. Have at least two reviewers discuss the Arabic terms and intended semantics. Choose a policy, record the reason, and encode it in negative compile tests before introducing functions.

### 2. Memory model experiment

Write two small programs that express ownership transfer, borrowing, aliasing and mutation. Compare a Rust-like ownership/borrowing model with an explicitly managed or runtime-checked alternative. Evaluate learnability in Arabic documentation, data-race guarantees, compile-time analysis, generated runtime burden, FFI and implementation complexity. Do not implement half of a borrow checker as a syntax feature; the model requires formal rules and dedicated tests.

### 3. IR and backend experiment

Define a small typed IR for literals, local storage, branching, loops, calls, returns and diagnostics. Build an IR validator and a readable IR dump first. Only then benchmark a bytecode interpreter, Cranelift and LLVM using the same valid and invalid test corpus. The experiment report must include target architecture, compile time, binary size, debugging, startup, and maintenance assumptions.

### 4. Bootstrap experiment

Specify exact reproducible commands and toolchain pins:
- Stage 0 compiles/executes the reference compiler implementation.
- Stage 1 is the same compiler written in Rukn, compiled by Stage 0.
- Stage 2 is compiled by Stage 1 and tested against Stage 1 behavior.
- Clean-build outputs are compared and all differences recorded.

This is evidence of a bootstrap process, not a mathematical proof of compiler correctness. Preserve Stage 0 as a recovery path while confidence in later stages grows.

## Acceptance gate for Rukn 0.1

Before tagging a public release:

- A reviewed versioned specification matches implementation and examples.
- All public features have valid and invalid test cases.
- Fuzz/property tests cover lexer and parser; malformed input must return diagnostics instead of panicking.
- Source-size, syntax-depth, text/output and step-budget limits are documented and regression-tested.
- Unicode policy includes tests for normalization, bidirectional controls, mixed scripts and known confusable cases.
- Formatting, Clippy with warnings denied, unit tests, CLI examples and all CI checks pass on the tagged commit.
- `Cargo.lock` is committed and dependency resolution runs with `--locked`.
- Changelog, release support policy, repository governance, issue templates, security reporting route, and contribution instructions are published.
- The project does not claim a native compiler, memory-safety guarantee, sandbox, or self-hosting until the corresponding acceptance gate passes.

## Reference standards and implementation material

- [Rust Compiler Development Guide — syntax and AST](https://rustc-dev-guide.rust-lang.org/syntax-intro.html)
- [Rust Compiler Development Guide — parser](https://rustc-dev-guide.rust-lang.org/the-parser.html)
- [Rust compiler bootstrapping](https://rustc-dev-guide.rust-lang.org/building/bootstrapping/what-bootstrapping-does.html)
- [Unicode UAX #31 — identifiers and syntax](https://www.unicode.org/reports/tr31/)
- [Unicode UAX #15 — normalization](https://www.unicode.org/reports/tr15/)
- [Unicode UAX #9 — bidirectional algorithm](https://www.unicode.org/reports/tr9/)
- [Unicode UTS #39 — security mechanisms](https://www.unicode.org/reports/tr39/)
- Competitive notes: [RESEARCH.md](RESEARCH.md)
