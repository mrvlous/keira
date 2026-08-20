<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Keira Kernel Userland & C SDK

Documentation for the Ring 3 userland execution environment, freestanding C runtime library (`libc`), and in-kernel KCC compiler toolchain.

## Documents

| Document | Description |
| :--- | :--- |
| **[`c_sdk.md`](c_sdk.md)** | Freestanding C Runtime Library (`stdio.h`, `stdlib.h`, `string.h`, `syscall.h`, `math.h`, `ctype.h`) |
| **[`kcc_compiler.md`](kcc_compiler.md)** | In-Kernel Modular KCC Compiler Binary (`/apps/bin/kcc.elf` in `user/bin/kcc/`) |
| **[`elf_execution.md`](elf_execution.md)** | Ring 3 Process Lifecycle, Memory Isolation, and Syscall Interface |
| **[`init.md`](init.md)** | User-space initialization sequence and startup runtime |
