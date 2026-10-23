# Contributing to Rukn

Rukn is at an early compiler-prototype stage. A small, reproducible improvement is more useful than a large feature that has no grammar, semantic contract or regression tests.

## Before proposing a language change

- Read [LANGUAGE-SPEC.md](LANGUAGE-SPEC.md) and [ARCHITECTURE.md](ARCHITECTURE.md).
- Describe the problem and provide at least one example program that demonstrates the intended behavior.
- Specify invalid cases and expected diagnostics, not only the successful example.
- Consider Arabic shaping, Unicode normalization, bidirectional display and keyboard entry.
- Keep source parsing deterministic; do not add natural-language guesses to the grammar.

## Implementation expectations

- Keep lexing, parsing, semantic checking, interpretation and CLI I/O separate.
- Add positive and negative tests for each syntax or semantic change.
- Avoid panics on user-controlled source input. Internal invariants should be explicit and tested.
- Check integer overflow and runtime limits; do not silently wrap.
- If a feature changes semantics, update the language specification and migration notes in the same change.
- Do not add dependencies without explaining why the standard library or an existing dependency is insufficient.
- Do not claim native compilation, ownership safety, sandboxing or self-hosting until those features exist and their acceptance tests pass.

## Checks

```sh
cargo fmt --manifest-path experiments/rukn/Cargo.toml -- --check
cargo clippy --manifest-path experiments/rukn/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path experiments/rukn/Cargo.toml
```

The prototype is currently located in `deku-system/experiments/rukn` as a temporary incubator. A dedicated repository should be created before a public alpha so language design, releases, issues and governance have a stable home.
