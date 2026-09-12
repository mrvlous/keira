<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Keira Kernel Userland Subsystems & C Toolchain

The `userland` documentation is hyper-modularized into 4 specialized domains covering the freestanding C SDK (`libc.a`), in-kernel KCC C compiler, dynamic ELF loader, and userland applications.

---

## Userland Submodules

```mermaid
graph TD
    Userland["Userland Subsystems"] --> SDK["sdk/<br/>Freestanding C Standard Library (libc.a)"]
    Userland --> Compiler["compiler/<br/>In-Kernel KCC C Compiler Internals"]
    Userland --> Loader["loader/<br/>Dynamic ELF64 Binary Loader & Process Memory"]
    Userland --> System["system/<br/>Users, Permissions, Hostname, Init & POSIX I/O"]
```

---

## Userland Module Index

| Submodule | Focus Area | Description |
| :--- | :--- | :--- |
| [`sdk/`](sdk/README.md) | Freestanding C SDK | Standard headers and archived static library (`libc.a`) |
| [`compiler/`](compiler/README.md) | Native KCC Compiler | Preprocessor, lexer tokenization, recursive descent parser, and code generator |
| [`loader/`](loader/README.md) | Dynamic ELF64 Loader | Program header validation, segment loading, address space setup, and rollback |
| [`system/`](system/README.md) | System Services & Apps | Ring 3 native applications (`kcc`, `sysinfo`), user database, and POSIX I/O |
