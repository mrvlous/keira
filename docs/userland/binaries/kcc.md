<!-- SPDX-License-Identifier: GPL-2.0-only -->

# `kcc.elf` Compiler Executable

`kcc` compiles C source code directly to freestanding Ring 3 ELF executables within the Keira environment.

---

## 1. Invocation Modes & CLI Reference

`kcc` follows standard UNIX C compiler ergonomics. Invoking `kcc` without arguments reports a fatal error instead of executing unintended builds:

```bash
keira:/# kcc
kcc: fatal error: no input files
compilation terminated.
```

### A. Help and Version Reference
```bash
keira:/# kcc -h
Usage: kcc [options] <source.c>

Description:
  Keira native freestanding C compiler toolchain.

Options:
  -o <path>      Specify output ELF binary (default: /bin/app.elf)
  -v, --version  Display compiler version
  -h, --help     Display this help reference and exit

keira:/# kcc -v
kcc (Keira C Compiler) 0.6.0
```

### B. Standard Compilation
```bash
# Compile C source into executable ELF binary:
keira:/# kcc /tmp/main.c -o /bin/app.elf
[INFO] Compiling source: /tmp/main.c -> /bin/app.elf
[DONE] Compilation Successful!
  Code size: 402 bytes, Data size: 69 bytes
  Functions compiled: 2
  Executable written to /bin/app.elf

# Execute the generated binary:
keira:/# /bin/app.elf
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
