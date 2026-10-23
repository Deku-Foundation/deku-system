# Rukn Stage-0 Language Specification

**Status:** prototype contract for the checked-in Stage-0 implementation, not a stable 1.0 specification. Grammar changes require updating parser tests and examples.

## 1. Source model

A source file is UTF-8 and is limited by the current implementation to 256 KiB and 2,048 lexer tokens. Syntax nesting for blocks, parenthesized expressions and chains of unary operators is limited to 128 levels. Line and column locations count Unicode code points starting from one; columns are not terminal display-cell measurements. Newlines separate simple statements. Blank lines and comments are ignored. A comment begins with `#` and continues to the end of that line.

Arabic is the primary keyword surface. The parser is deterministic and does not infer intent from natural-language phrasing. Operators and grouping punctuation remain conventional ASCII characters; the Arabic comma `،` is accepted by the lexer, although the Stage-0 grammar has no comma-separated constructs.

## 2. Tokens and identifiers

Identifiers use Unicode XID start/continue properties, permit underscore, and are compared after NFC normalization. They are case-sensitive. NFC is canonical normalization; the implementation does not strip Arabic diacritics or treat hamza variants as interchangeable. Keywords are reserved after normalization.

The keywords are:

| Keyword | Meaning |
|---|---|
| `دع` | Declare a mutable binding |
| `ثابت` | Declare an immutable binding |
| `اطبع` | Print a value |
| `إذا` | Begin a conditional |
| `وإلا` | Begin the alternative branch |
| `طالما` | Begin a while loop |
| `نهاية` | End a conditional or loop |
| `صح`, `خطأ` | Boolean literals |
| `و`, `أو`, `ليس` | Boolean conjunction, disjunction, and negation |

A Stage-0 integer literal can use ASCII digits (`0-9`), Arabic-Indic digits (`٠-٩`), or Eastern Arabic/Persian digits (`۰-۹`). A literal must use one set consistently. Underscores may separate digits. Values are signed 64-bit integers, but the unsigned magnitude `2^63` cannot currently be written as a literal, so the minimum signed integer is not expressible directly. This limitation must be addressed before integer literals are stabilized.

String literals use double quotes. Escapes currently supported are `\\n`, `\\r`, `\\t`, `\\"` and `\\\\`. Raw newlines inside a string are rejected.

## 3. Statements

```ebnf
program         = { newline | statement, { newline } } ;
statement       = declaration
                | assignment
                | print
                | if_statement
                | while_statement ;

declaration     = ("دع" | "ثابت"), identifier, "=", expression ;
assignment      = identifier, "=", expression ;
print           = "اطبع", "(", expression, ")" ;
if_statement    = "إذا", expression, newline,
                  block,
                  [ "وإلا", newline, block ],
                  "نهاية" ;
while_statement = "طالما", expression, newline, block, "نهاية" ;
block           = { newline | statement, { newline } } ;
```

The EBNF describes structure. The implementation also requires each simple statement to end at a newline or end of file. Blocks may contain empty lines. Stage 0 has no functions, `return`, imports, user-defined types or comma-separated declarations.

## 4. Expressions and precedence

From highest to lowest:

1. Parentheses, literals, variable references.
2. Unary `-` and `ليس`.
3. `*`, `/`, `%`.
4. `+`, `-`.
5. `<`, `<=`, `>`, `>=`.
6. `==`, `!=`.
7. `و`.
8. `أو`.

Binary operators at the same precedence level associate left-to-right. Boolean `و` and `أو` short-circuit.

## 5. Types and scope

Stage 0 infers the type of a declaration from its initializer. Available types are integer, text and Boolean. Mutable bindings cannot change type. Constants cannot be assigned after declaration. A type checker runs before execution, so undefined names, duplicate declarations in one scope, assignment to a constant, incompatible operators and non-Boolean conditions are rejected before the program starts.

Conditional and loop bodies introduce lexical scopes. Bindings declared within a block are not visible after the block. Assignment can update an outer mutable binding when the name resolves to that binding.

## 6. Arithmetic and runtime behavior

Integer addition, subtraction, multiplication, unary negation, division and remainder use checked 64-bit operations. Overflow and division by zero produce an error rather than wrapping or panicking. Comparisons operate on integers. Equality operates on values of the same type; values of different types compare unequal.

The interpreter has a default step budget of 1,000,000 statement/loop steps, caps individual text values at 256 KiB and total printed output at 1 MiB. This prevents accidental infinite loops in the prototype; it is not a sandbox or a security boundary. Printed Boolean values are `صح` and `خطأ`.

## 7. Diagnostics

Diagnostics identify the phase (lexing, parsing, semantic checking, runtime or I/O), the source line and code-point column, and a message in Arabic. Diagnostics are part of the user-facing interface: changing a diagnostic should have a regression test when the distinction matters.

## 8. Compatibility policy

Until version 1.0, changes may be breaking. A stable specification must define lexical normalization, numeric range, type inference, integer overflow, evaluation order, scope, diagnostics and binary interoperability before compatibility promises are made.
