# Rukn research and positioning

**Research date:** 2026-10-23. This is a snapshot of public project descriptions, not an audit of their source correctness, license obligations or release quality. Feature lists below are attributed claims; they should not be treated as independently verified.

## Nearby projects

| Project | Publicly described direction | Implication for Rukn |
|---|---|---|
| [SultanC](https://github.com/sultan-language/sultanc) | An Arabic-first systems language with Arabic/English syntax surfaces, self-hosting, ownership concepts, generics, an interpreter and native targets. Public topic information and a recent author announcement describe its direction. | Arabic syntax plus native compilation and self-hosting is not an unoccupied niche. Rukn needs a sharper, testable contribution and should not copy its syntax or branding. |
| [Baa](https://github.com/OmarAglan/Baa) | An Arabic-syntax compiled systems-language project. Its public repository includes a compiler, examples, standard-library work, an intermediate representation and a substantial roadmap. | Study its published grammar, diagnostics and target decisions before Rukn freezes a design. Avoid parallel feature claims without measurable evidence. |
| [Sad Language](https://www.sad-lang.org/en/) | Its website describes an LLVM-based native compiler, Arabic keywords, a standard library, package manager and language-server tooling. | A language name alone is not differentiation. Maintain a public conformance suite and describe implemented functionality more precisely than promotional feature lists. |
| [Dhad (ضاد)](https://github.com/maljefairi/dhad) | A declarative Arabic language with a YAML/Rasa-style structure and teaching-oriented execution narration. | A useful model for beginner diagnostics and explanations, but it is a different design point from a future systems language. Rukn should keep a deterministic textual grammar and explicit static semantics. |
| [Alang / ArabicLang](https://github.com/adamgaafar/ArabicLang) | An experimental Java/ANTLR interpreter with Arabic keywords and identifiers. | Lexer, parser and interpreter examples are useful, but Rukn should establish source locations, tests and a formal grammar rather than stop at an educational demonstration. |
| [Nazm](https://github.com/OmarAglan/Nazm) | An Arabic assembler for x86-64, with explicit Unicode and integration documentation. | A potential future integration and learning reference for assembly work; Rukn should not invent an assembly layer before its IR and ABI are specified. |

The [GitHub Arabic programming-language topic](https://github.com/topics/arabic-programming-language) is a useful discovery point, but topics and repository descriptions are incomplete and may be outdated. Keep a dated research note rather than assuming the landscape stands still.

## Name collision review

“Rukn” (رُكن) is a meaningful and pronounceable name, but it is not unique. The public [Rukn Design System repository](https://github.com/mfaizanatiq/rukn-design-system) uses the name for a multilingual design system, and a [RUKN GitHub organization](https://github.com/ruknsoftware) also exists in the software space. This does **not** establish a trademark conflict. It does establish discoverability and namespace-collision risk.

Recommendation for now: use the descriptive public name **Rukn Language — ركن**, with the steward shown as **Deku Foundation**. Before a 1.0 launch, check GitHub repository availability, package registries, domains, software catalogs and relevant trademark registers. Do not assert legal clearance from a web search alone.

## Recommended differentiation

Rukn should compete on a small set of measurable engineering commitments:

1. **Arabic is part of the language specification.** Identifier normalization, bidirectional controls, number forms, source spans and diagnostics are specified and tested—not handled as a visual afterthought.
2. **Static semantics precede feature count.** Types, scopes, evaluation order, overflow and error handling have a normative contract and negative tests.
3. **One semantic engine.** The CLI, formatter, IDE integrations and future language server reuse the same lexer/parser/type checker.
4. **Evidence-based bootstrap.** Stage 0, Stage 1 and later compiler stages are explicit. A “self-hosting” label is withheld until a Rukn compiler actually compiles its own source under a tested bootstrap process.
5. **Reproducibility and truthful status.** A green CI run is tied to the exact commit; compiler features are marked implemented only when tests demonstrate them.

These choices are process and architecture commitments, not claims that Rukn is already more capable than the projects above.

## Primary technical references

- [Rust Compiler Development Guide: syntax and AST](https://rustc-dev-guide.rust-lang.org/syntax-intro.html) — separates source processing, parsing and later compiler phases.
- [Rust Compiler Development Guide: lexing and parsing](https://rustc-dev-guide.rust-lang.org/the-parser.html) — tokens, source locations and syntax trees.
- [Rust compiler bootstrapping](https://rustc-dev-guide.rust-lang.org/building/bootstrapping/what-bootstrapping-does.html) — staged compiler builds and stage comparison.
- [Unicode UAX #31](https://www.unicode.org/reports/tr31/) — identifier and syntax rules.
- [Unicode UAX #15](https://www.unicode.org/reports/tr15/) — normalization.
- [Unicode UAX #9](https://www.unicode.org/reports/tr9/) — bidirectional text layout.
- [Unicode UTS #39](https://www.unicode.org/reports/tr39/) — spoofing and confusable-character mechanisms.
