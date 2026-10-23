# Rukn Stage-0 security notes

This is a language implementation prototype. It is not a sandbox. A Rukn program can consume the interpreter's step budget and print output, but the current language has no filesystem, network, subprocess or FFI operations. Do not treat that absence as a sandbox guarantee for future versions, and do not run untrusted programs under elevated privileges.

## Source-text security

Arabic and mixed-direction text need explicit lexical rules. Rukn currently:

- Requires UTF-8 source.
- Uses Unicode XID start/continue properties for identifiers, with underscore allowed.
- Normalizes identifiers to NFC before keyword recognition and name comparison.
- Does not remove diacritics or apply fuzzy equivalences between Arabic letters.
- Rejects selected bidirectional formatting and invisible control characters in source, including comments and string literals.
- Rejects mixing ASCII, Arabic-Indic and Eastern Arabic/Persian digit sets inside one integer token.
- Reports source locations in code points; editors must not treat them as visual display-cell offsets.

These rules are a starting point, not complete spoof detection. A future release should evaluate warnings for confusable identifiers and mixed-script names using Unicode Technical Standard #39, and include editor snapshots to test what a reviewer sees versus the actual code-point sequence.

Normative references:

- [Unicode Standard Annex #31 — Identifiers and Syntax](https://www.unicode.org/reports/tr31/)
- [Unicode Standard Annex #15 — Normalization Forms](https://www.unicode.org/reports/tr15/)
- [Unicode Standard Annex #9 — Bidirectional Algorithm](https://www.unicode.org/reports/tr9/)
- [Unicode Technical Standard #39 — Unicode Security Mechanisms](https://www.unicode.org/reports/tr39/)

## Runtime boundaries

- The lexer rejects source files above 256 KiB and more than 2,048 tokens; the parser limits block, parenthesized-expression and unary-operator nesting to 128 levels.
- The interpreter has a fixed default step budget to stop infinite loops, caps individual text values at 256 KiB and total printed output at 1 MiB; it does not enforce a wall-clock timeout or a general process memory limit.
- Integer operations are checked. Overflow and division by zero return diagnostics.
- Static checking rejects undefined identifiers, incompatible types, invalid Boolean conditions and assignments to constants before execution.
- Programs currently cannot open files, use the network, spawn processes or call native code.
- Future I/O and FFI must receive an explicit capability and trust design; do not add these features as arbitrary host calls.

## Supply-chain and build controls

- Pin the Rust toolchain used for official releases.
- The incubator now commits `Cargo.lock`, and Rukn CI uses `--locked` for Clippy, tests and example execution. Review dependency updates deliberately.
- Run formatting, Clippy with warnings denied, unit tests, example-output tests and fuzz/property tests as the parser grows.
- Publish checksums and attestations for release artifacts; a checksum alone is not publisher authentication.
- Keep code-generation and release-signing credentials outside untrusted build steps.

## Reporting

Until a dedicated Rukn repository and private reporting route exist, report language implementation issues in the [Deku Foundation repositories](https://github.com/Deku-Foundation) without publishing secrets or exploit details for any future privileged feature.
