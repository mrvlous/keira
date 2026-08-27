<!-- SPDX-License-Identifier: GPL-2.0-only -->

# KCC Lexical Tokenizer (`lexer.c`)

This document details source text tokenization, keyword identification, and literal parsing in the KCC C compiler.

---

## Token Kinds

| Token Kind | Example | Description |
| :--- | :--- | :--- |
| `TOKEN_KEYWORD` | `int`, `return`, `if`, `while`, `for` | Reserved C language control and type keywords |
| `TOKEN_IDENTIFIER`| `main`, `count`, `buffer` | User-defined variable and function symbols |
| `TOKEN_NUMBER` | `42`, `0x1000` | Decimal and hexadecimal numeric literals |
| `TOKEN_STRING` | `"Hello, Keira!\n"` | Null-terminated string literals |
| `TOKEN_OPERATOR` | `+`, `-`, `*`, `/`, `==`, `!=` | Arithmetic, relational, and logical operators |
