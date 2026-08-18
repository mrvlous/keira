<!--
SPDX-License-Identifier: GPL-2.0-only

Keira Kernel - Operating System Kernel
Copyright (C) 2026 Moh. Ananda Firmansyah Putra
-->

# ELF64 Binary Loader and Userland Execution

## Overview

The Keira Kernel provides an isolated userland execution environment for 64-bit ELF (Executable and Linkable Format) binaries running in CPU Ring 3 (User Mode). The ELF loader validates headers, allocates address space page tables, loads program segments, and initializes the user-space stack.

```
+-------------------------------------------------------------+
|                      Keira Userland ELF                     |
+-------------------------------------------------------------+
| Virtual Address Range     | Description                     |
+---------------------------+---------------------------------+
| 0x0000000040000000        | Code & Data (.text, .rodata)    |
| 0x0000000040001000        | Initialized & BSS Data (.data)  |
| 0x0000600000000000        | Dynamic Heap Memory (sbrk/malloc) |
| 0x00007FFFFFFF0000        | User Stack Frame (4KB - 2MB)    |
+-------------------------------------------------------------+
```

## Binary Header Verification

The loader strictly validates ELF64 executables before mapping:
* **Magic Number**: `0x7F 'E' 'L' 'F'`
* **Class**: 64-bit (`ident[4] == 2`)
* **Endianness**: Little-Endian (`ident[5] == 1`)
* **Machine Type**: x86_64 (`e_machine == 0x3E`)
* **Segment Type**: `PT_LOAD` (Loadable code/data)

## Address Space Isolation

Each userland process executes within an isolated Page-Map Level-4 (PML4) paging structure:
1. `vmm::clone_kernel_pml4()` clones lower 1GB identity map into child PDPT while keeping user space isolated.
2. `vmm::map_page()` assigns `PAGE_USER | PAGE_WRITABLE | PAGE_PRESENT` attributes to application segments.
3. User stack frame is allocated at `0x7FFFFFFF0000`.
4. Execution transitions to userland via `IRETQ` with Ring 3 Code Segment (`0x28 | 3 = 0x2B`) and User Stack Segment (`0x20 | 3 = 0x23`).
5. Fast system calls execute via `syscall` / `sysretq` using STAR MSR (`0x00180008`) and LSTAR MSR (`syscall_handler_asm`).

## Available Userland Binaries

* `/apps/bin/user_test.elf`: System verification init process.
* `/apps/bin/gcc.elf`: Freestanding C compiler environment test.
* `/apps/bin/cat.elf`: VFS file reading and printing utility.
* `/apps/bin/ping.elf`: ICMP Echo Request network diagnostic utility.
* `/apps/bin/echo.elf`: Userland string output process.
* `/apps/bin/uptime.elf`: Hardware RTC and timer reader.
