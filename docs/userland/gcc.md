# Self-Hosting C Compiler (gcc)

This document details the lexical analysis, parsing, and execution architecture of the built-in self-hosting C compiler (`bin/gcc`) included in Keira Kernel.

## 1. Compiler Overview
The self-hosting compiler ([gcc.c](../../user/bin/gcc.c)) is a custom compiler designed to compile C source files directly in the user-space environment of Keira Kernel (v0.11.0). It links against the extended `libc` runtime (`stdio.h`, `stdlib.h`, `string.h`, `syscall.h`, `socket.h`, `math.h`, `time.h`, `malloc.h`).

---

## 2. Lexical Analyzer (Lexer)
The lexer scans the raw input character buffer to extract lexical tokens.

### Token Types
The lexer identifies the following token structures:
*   **Keywords**: `int`, `char`, `void`, `main`, `printf`, `return`, `if`, `else`, `while`, `for`, `syscall`.
*   **Identifiers**: User-defined variable names, global symbols, and function names.
*   **Literals**: Integer numbers, character literals, and string constants.
*   **Operators and Punctuators**: `+`, `-`, `*`, `/`, `=`, `==`, `!=`, `<`, `>`, `<=`, `>=`, `&&`, `||`, `;`, `,`, `{`, `}`, `(`, `)`, `[`, `]`.

### Lexer Implementation
*   `skip_whitespace()`: Skips spaces, tabs, newlines, and preprocessor directives (lines starting with `#`).
*   `next_token()`: Scans the input stream, extracts tokens into `token_string` or `token_num`, and returns the corresponding integer token ID.
*   `match(expected)`: Asserts that the current token matches `expected` and advances the token stream, aborting on syntax errors.

---

## 3. Syntax Analyzer (Parser) and Control Flow
The parser processes token sequences using single-pass recursive descent routines:
*   **Statement Parsing (`statement`)**: Parses statement blocks (`{ ... }`), local variable declarations (`int x;`, `char *p;`), assignments (`x = expr;`), pointer dereferences (`*p = val;`), system call invocations (`syscall(num, a1, a2, a3)`), formatted output (`printf("...")`), conditional branches (`if (cond) stmt1 else stmt2`), and loops (`while (cond) stmt`, `for (init; cond; post) stmt`).
*   **Expression Parsing (`expression`, `add_expr`, `mul_expr`, `primary_expr`)**: Precedence-driven recursive descent for binary operations:
    1. Primary: Numbers, string constants, variable lookups (`lookup_local`, `lookup_global`), pointer dereferencing, system calls.
    2. Multiplicative: `*`, `/`.
    3. Additive: `+`, `-`.
    4. Relational: `<`, `>`, `<=`, `>=`, `==`, `!=`.

---

## 4. Code Generation & ELF64 Executable Emission
The compiler generates x86_64 machine code directly into `code_buf` and static data into `data_buf`:
1.  **Register Convention**: `RAX` serves as the primary accumulator for expression evaluation and return values; `RCX` and `RSI` serve as secondary registers.
2.  **Stack Frame Allocation**: Local variables are allocated on the stack relative to `RBP` (`-8`, `-16`, etc.) using `lookup_local` and `add_local`.
3.  **Symbol & Jump Patching**: `function_addresses`, `patch_addresses`, and `val_patch_addresses` track forward function calls and data section references, patching relative offsets (`E9` / `0F 84` jumps) in a final emission pass.
4.  **ELF Header Generation**: `write_elf` wraps `code_buf` and `data_buf` with standard ELF64 header headers (entry point, loadable segments `PT_LOAD`, segment permissions), emitting a standalone ELF64 user-space binary.
