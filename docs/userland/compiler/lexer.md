<!-- SPDX-License-Identifier: GPL-2.0-only -->

# KCC Lexical Analyzer (`userland/bin/kcc/lexer/`)

Converts preprocessed C source text into a stream of typed tokens.

---

## Token Categories

* **Keywords**: `int`, `char`, `void`, `return`, `if`, `else`, `while`, `for`, `struct`, `union`, `typedef`, `sizeof`.
* **Identifiers**: Variable and function names conforming to `[a-zA-Z_][a-zA-Z0-9_]*`.
* **Literals**: Decimal, hexadecimal (`0x...`), octal integers; character constants (`'a'`); string literals (`"..."`).
* **Operators**: Arithmetic (`+`, `-`, `*`, `/`, `%`), bitwise (`&`, `|`, `^`, `~`, `<<`, `>>`), relational (`==`, `!=`, `<`, `<=`, `>`, `>=`), logical (`&&`, `||`, `!`), and assignment (`=`, `+=`, etc.).
