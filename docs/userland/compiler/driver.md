<!-- SPDX-License-Identifier: GPL-2.0-only -->

# KCC Compiler Driver & CLI Interface (`main.c`)

This document details compiler driver flags, file inputs, and binary generation in the KCC C compiler.

---

## Command Line Syntax

```bash
run /system/bin/kcc.elf [options] <source.c> -o <output.elf>
```

### Supported Flags:
* `-o <file>`: Specify output binary filename.
* `-v`: Display compiler version and target architecture information.
* `-c`: Compile and assemble only; do not link.
* `-I<dir>`: Add include directory to preprocessor header search path.
