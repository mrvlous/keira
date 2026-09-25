<!-- SPDX-License-Identifier: GPL-2.0-only -->

# `kcc.elf` Compiler Executable

`kcc` is invoked from the Keira shell to compile C source code directly to ELF executables.

---

## Command Usage

```bash
kcc [-o output.elf] [-c] [-I include_dir] source.c
```

---

## Execution Flow

1. Parses command line flags using standard `getopt`.
2. Invokes the preprocessor on the input source file.
3. Generates the AST, executes semantic validation and symbol resolution.
4. Generates x86 machine code and packages output as an executable ELF binary.
5. Sets executable file permissions (`0755`) on the target file.
