# Rukn (رُكن) — Stage-0 language prototype

Rukn is an experimental Arabic-first programming language being developed under the Deku Foundation. The name refers to a pillar or supporting foundation. Its long-term goal is a statically checked, compiled language whose compiler and developer tools can eventually be implemented in Rukn itself.

**Current status:** a small interpreter implemented in Rust. It is not yet a native compiler, a self-hosted compiler, a systems-programming language with ownership, or a production toolchain.

This is a temporary incubator inside `deku-system` while the project establishes separate language governance. Rukn should move to a dedicated repository such as `Deku-Foundation/rukn-lang` before the first public alpha; the interpreter must not remain permanently coupled to system-image assembly.

## Try it

Install a stable Rust toolchain (Rust 1.85 or newer), then run these commands from the repository root:

```sh
cargo test --manifest-path experiments/rukn/Cargo.toml
cargo run --manifest-path experiments/rukn/Cargo.toml -- check experiments/rukn/examples/مرحبا.rukn
cargo run --manifest-path experiments/rukn/Cargo.toml -- run experiments/rukn/examples/مرحبا.rukn
```

Expected output from the example:

```text
مرحبًا بك في ركن
ناتج الحساب:
12
42
```

The test suite and the executable are the source of truth for what this prototype currently supports.

## Current language surface

- Arabic keywords and Unicode identifiers.
- Arabic-Indic digits (٠-٩), Eastern Arabic/Persian digits (۰-۹), or ASCII digits; a single numeric literal may not mix digit systems.
- Integer, string and Boolean literals.
- Mutable `دع` declarations and immutable `ثابت` declarations.
- Arithmetic, comparisons, equality, `و`, `أو`, and `ليس`.
- `اطبع(...)`, `إذا ... / وإلا / نهاية`, and `طالما ... / نهاية`.
- A static semantic pass for inferred integer, string and Boolean types.
- UTF-8 source, NFC-normalized identifiers, and rejection of selected invisible direction/format controls.
- Arabic diagnostics with source line and column, bounded source/token/parse-depth limits, checked arithmetic, bounded text/output sizes and a step count.

## Deliberate non-goals for Stage 0

This prototype does not implement functions, user-defined types, modules, arrays, generics, explicit type annotations, ownership/borrowing, package management, a language server, a bytecode VM or native code generation. It does not claim memory safety for programs, sandboxing, ABI stability, or self-hosting. The Rust host implementation uses ordinary process privileges; do not run untrusted source as though it were sandboxed.

## Engineering rules

1. Treat the parser and type checker as language implementation components, not as a natural-language parser.
2. A language feature is complete only when its grammar, semantics, diagnostics and regression tests are updated together.
3. Do not silently accept ambiguous Unicode, numeric overflow or type errors.
4. Do not promise a native backend before its ABI, runtime and test matrix are specified.
5. Keep the Rust Stage-0 implementation small enough to audit, and make every later bootstrap stage reproducible.

See the [language specification](LANGUAGE-SPEC.md), [design-decision register](DESIGN-DECISIONS.md), [architecture](ARCHITECTURE.md), [Unicode security notes](SECURITY.md), [competitive research](RESEARCH.md), [contribution guide](CONTRIBUTING.md), and [roadmap](ROADMAP.md).
