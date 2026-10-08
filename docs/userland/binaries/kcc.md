<!-- SPDX-License-Identifier: GPL-2.0-only -->

# `kcc.elf` Compiler Executable

`kcc` compiles C source code directly to freestanding Ring 3 ELF executables within the Keira environment.

---

## 1. Invocation Modes

### A. Native Shell Command
```bash
kcc /tmp/main.c -o /bin/app.elf
```

### B. Freestanding Userland ELF
```bash
run /bin/kcc.elf /tmp/main.c -o /bin/app.elf
```

---

## 2. Compilation & Execution Workflow

1. **Preprocessing**: Resolves standard `#include` headers from `/include/` and source directories.
2. **Lexing & Parsing**: Generates the Abstract Syntax Tree (AST), performs symbol resolution and allocates stack frames.
3. **Machine Code Generation**: Emits target architecture instructions (`x86_64` or `i686`).
4. **ELF Packaging**: Encapsulates executable code into a valid 64-bit/32-bit ELF binary with standard program headers.
5. **Execution**: The compiled binary can be launched immediately in an isolated address space:
   ```bash
   run /bin/app.elf
   ```
