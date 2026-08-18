<!--
SPDX-License-Identifier: GPL-2.0-only

Keira Kernel - Operating System Kernel
Copyright (C) 2026 Moh. Ananda Firmansyah Putra
-->

# Userland Subsystems & Toolchain

Welcome to the Userland documentation section for Keira Kernel.

## Documents

* [User Runtime Library (libc & Extensions)](runtime.md): Dynamic memory allocation (malloc), POSIX stdio file I/O, environment variables, socket programming (`socket.h`), C Math (`math.h`) & Time (`time.h`), and system call wrappers.
* [ELF64 Binary Loader & Userland Execution](elf_loader.md): Ring 3 user mode execution, PML4 address space isolation, segment mapping, and stack frame initialization.
* [POSIX File Descriptors & Stream I/O](posix_io.md): Standard POSIX file descriptor table (0..1024), access mode flags (`fcntl.h`), and stream system call vectors.
* [Multi-User Account Management](users.md): Persistent user management (`user`), password storage (`/system/etc/passwd`), dynamic prompt, 3-attempt retry fallback, and UNIX privilege separation.
* [System Hostname Configuration](hostname.md): System hostname configuration (`hostname`) persisted to `/system/etc/hostname`.
* [POSIX File Security & Attributes](permissions.md): POSIX file security & protection flags (`protect`, `fileinfo`).
* [Multi-Virtual Terminal Subsystem](tty.md): Virtual terminal switching (`tty1`..`tty4`) and console screen buffers.
* [The Init Process](init.md): User-space initialization sequence (`bin/init`) spawning system processes.
* [Self-Hosting C Compiler](gcc.md): Parser, lexer, AST builder, and helper structures inside the built-in C compiler (`bin/gcc`).
