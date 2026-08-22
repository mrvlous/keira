<!-- SPDX-License-Identifier: GPL-2.0-only -->

# In-Kernel KCC Compiler Toolchain

Keira Kernel includes a native freestanding in-kernel C compiler executable (`/apps/bin/kcc.elf`) organized in a hyper-modular architecture under `user/bin/kcc/`.

## Modular Architecture

```text
user/bin/kcc/
├── include/              # Private compiler declarations
│   ├── common.h          # Global compiler state, buffers & diagnostic helpers
│   ├── lexer.h           # Token types (TOK_*), scanner variables & token prototypes
│   ├── symbols.h         # Global/local symbol tables, address resolution & loop stacks
│   ├── codegen.h         # x86_64 machine code generator & System V AMD64 ABI declarations
│   ├── parser.h          # Recursive descent expression & statement parser
│   └── elf.h             # ELF64 binary generator prototypes
├── common.c              # Buffer allocations & diagnostic helpers
├── lexer.c               # Character scanner, string/number literals & line tracking
├── symbols.c             # Variable offsets, function addresses & relocation tables
├── codegen.c             # x86_64 instruction encoder, register management & jumps
├── parser.c              # AST parsing, operator precedence & control flow generation
├── elf.c                 # ELF64 header, segment builder & binary writer
└── main.c                # Compiler entry point (`_start`), file I/O & pipeline driver
```

## Compilation Pipeline

```mermaid
graph TD
    A["/data/main.c (Source Code)"] --> B["lexer.c (Token Stream & Line Tracking)"]
    B --> C["parser.c (Recursive Descent Parser)"]
    C --> D["symbols.c (Symbol & Scope Resolution)"]
    C --> E["codegen.c (x86_64 Machine Code Emitter)"]
    D --> E
    E --> F["elf.c (ELF64 Executable Packaging)"]
    F --> G["/apps/bin/app.elf (Executable Output)"]
```

## Supported Language Features & Syntax

| Category | Supported Syntax |
| :--- | :--- |
| **Data Types** | `int`, `char`, `short`, `long`, `unsigned`, `signed`, `void`, pointers (`*`) |
| **Arithmetic** | `+`, `-`, `*`, `/`, `%` (modulo) |
| **Bitwise** | `&` (AND), `|` (OR), `^` (XOR), `~` (NOT), `<<` (SHL), `>>` (SHR) |
| **Logical** | `&&` (AND), `||` (OR), `!` (NOT) |
| **Relational** | `<`, `>`, `<=`, `>=`, `==`, `!=` |
| **Assignments** | `=`, `+=`, `-=`, `*=`, `/=`, `%=`, `&=`, `|=`, `^=`, `<<=`, `>>=` |
| **Increment/Decrement** | `++var`, `--var`, `var++`, `var--` |
| **Pointers & Memory** | `&var` (address-of), `*ptr` (read/write dereference), `arr[i]` (byte/index addressing) |
| **Control Flow** | `if` / `else`, `while`, `for`, `do ... while`, `break;`, `continue;`, `return` |
| **ABI & Calls** | System V AMD64 ABI (up to 6 parameters: `rdi`, `rsi`, `rdx`, `rcx`, `r8`, `r9`), `syscall(...)` |
| **Built-ins** | `sizeof(type\|var)`, `printf("format", ...)` |

## Compiler Execution

1. Place C source code into `/data/main.c` (or `/temp/main.c`):
   ```c
   #include <stdio.h>
   #include <syscall.h>

   int fib(int n) {
       if (n <= 1) return n;
       return fib(n - 1) + fib(n - 2);
   }

   void main(void) {
       printf("Keira KCC Compiler Demo\n");
       int i;
       for (i = 0; i < 10; i++) {
           printf("Fibonacci calculation complete.\n");
       }
   }
   ```
2. Execute the native compiler from the Keira shell:
   ```bash
   run /apps/bin/kcc.elf
   ```
3. Execute the resulting compiled ELF64 binary:
   ```bash
   run /apps/bin/app.elf
   ```
