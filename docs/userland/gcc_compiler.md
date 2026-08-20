<!-- SPDX-License-Identifier: GPL-2.0-only -->

# In-Kernel GCC Compiler Toolchain

Keira Kernel includes a freestanding C compiler executable (`/apps/bin/gcc.elf`).

## Features
- Compiles standard C source files (e.g. `/data/main.c`) directly into 64-bit ELF binaries (`/apps/bin/app.elf`) inside the running kernel environment.
- Invoked via the native shell command: `run /apps/bin/gcc.elf`.
