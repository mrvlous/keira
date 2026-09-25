<!-- SPDX-License-Identifier: GPL-2.0-only -->

# KCC Recursive Descent Parser (`userland/bin/kcc/parser/`)

Transforms token streams into an Abstract Syntax Tree (AST) representing functions, control structures, and expressions.

---

## Expression Parsing (Precedence Climbing)

Operator precedence is resolved using Pratt / precedence climbing parsing:
1. Primary expressions (literals, variables, parenthesized sub-expressions).
2. Postfix operators (function calls `f()`, array indexing `a[i]`, member access `.`, `->`).
3. Unary operators (`*`, `&`, `-`, `!`, `~`, `sizeof`).
4. Multiplicative (`*`, `/`, `%`), Additive (`+`, `-`).
5. Shift (`<<`, `>>`), Relational, Equality, Bitwise, Logical, Conditional (`?:`), Assignment.
