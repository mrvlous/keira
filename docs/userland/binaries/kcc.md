<!-- SPDX-License-Identifier: GPL-2.0-only -->

# `kcc.elf` Compiler Executable

`kcc` compiles C source code directly to freestanding Ring 3 ELF executables within the Keira environment.

---

## 1. Invocation Modes

### A. Native Shell Command
```bash
kcc /data/main.c -o /apps/bin/app.elf
```

### B. Freestanding Userland ELF
```bash
run /system/bin/kcc.elf /data/main.c -o /apps/bin/app.elf
```

---

## 2. Compilation & Execution Workflow

1. **Preprocessing**: Resolves standard `#include` headers from `/system/include/` and source directories.
2. **Lexing & Parsing**: Generates the Abstract Syntax Tree (AST), performs symbol resolution, and allocates stack frames.
3. **Machine Code Generation**: Emits target architecture instructions (`x86_64` or `i686`).
4. **ELF Packaging**: Encapsulates executable code into a valid 64-bit/32-bit ELF binary with standard program headers.
5. **Execution**: The compiled binary can be launched immediately in an isolated address space:
   ```bash
   run /apps/bin/app.elf
   ```
