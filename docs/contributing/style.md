<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Coding Style & Documentation Standards

This document establishes the official coding, documentation, licensing, and commit conventions for Keira Kernel.

---

## 1. License Header Policy

Every source file (`.rs`, `.c`, `.h`, `.asm`, `.inc`), build script (`Makefile`), configuration file (`.toml`, `.json`), and documentation file (`.md`) **MUST** begin with a standard GPL-2.0-only license header.

> [!IMPORTANT]
> **Do NOT include author email addresses in file headers.**

### Rust (`.rs`):
```rust
// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.
```

### C & Headers (`.c`, `.h`):
```c
/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Operating System Kernel
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */
```

### Assembly (`.asm`, `.inc`):
```nasm
; SPDX-License-Identifier: GPL-2.0-only
;
; Keira Kernel - Operating System Kernel
; Copyright (C) 2026 Moh. Ananda Firmansyah Putra
;
; This program is free software; you can redistribute it and/or modify
; it under the terms of the GNU General Public License as published by
; the Free Software Foundation; version 2 of the License.
```

### Makefile, Shell, TOML (`Makefile`, `.sh`, `.toml`):
```makefile
# SPDX-License-Identifier: GPL-2.0-only
#
# Keira Kernel - Operating System Kernel
# Copyright (C) 2026 Moh. Ananda Firmansyah Putra
#
# This program is free software; you can redistribute it and/or modify
# it under the terms of the GNU General Public License as published by
# the Free Software Foundation; version 2 of the License.
```

### Markdown (`.md`):
```markdown
<!-- SPDX-License-Identifier: GPL-2.0-only -->
```

---

## 2. Rust Conventions

1. **Naming Conventions**:
   - Modules, functions, and variables: `snake_case`.
   - Structs, enums, traits, and type aliases: `UpperCamelCase`.
   - Global constants and static variables: `SCREAMING_SNAKE_CASE`.
2. **Defensive Kernel Programming**:
   - Pure `#![no_std]` environment across all crates.
   - Avoid `unwrap()` and `panic!` in kernel-space paths. Prefer `Result<T, &'static str>` or custom error enums.
   - Every `unsafe` function and block must provide a formal `# Safety` docstring explaining preconditions.
3. **Documentation**:
   - Write clear, formal English rustdoc comments (`///`) on all public types, constants, and functions.
   - Provide file-level module docstrings (`//!`) at the top of every module.
4. **Code Formatting**:
   - Always run `cargo fmt --all` before committing (enforced via `make format`).

---

## 3. C & Assembly Conventions

1. **Userland C (`user/`)**:
   - Follow standard Linux kernel C conventions (4-space indentation, braces on new lines for functions).
   - Use freestanding headers and standard type definitions from `<stdint.h>`, `<stddef.h>`, and `<stdbool.h>`.
   - Protect all header files with standard header guards:
     ```c
     #ifndef _KEIRA_HEADERNAME_H
     #define _KEIRA_HEADERNAME_H
     ...
     #endif /* _KEIRA_HEADERNAME_H */
     ```
   - Run `clang-format` and `clang-tidy` (enforced via `make format` and `make lint`).
2. **Low-Level Assembly (`arch/x86/`)**:
   - Use standard NASM x86_64 syntax.
   - Use semicolon (`;`) for comments.
   - Maintain 16-byte stack alignment across all interrupt service routines and context switches.

---

## 4. Filesystem & Path Standards

All runtime and VFS paths must adhere to the canonical 6-directory hierarchy:

* `/system`: Core binaries (`/system/bin`), device nodes (`/system/dev`), drivers (`/system/drivers`), and headers (`/system/include`).
* `/apps`: Userland executables (`/apps/bin`) and C source samples (`/apps/src`).
* `/config`: Boot config (`/config/boot`) and system configurations (`/config/sys`).
* `/users`: Multi-user home folders (`/users/admin`, `/users/default`, `/users/guest`).
* `/data`: Persistent templates and diagnostic logs (`/data/main.c`, `/data/log/`).
* `/temp`: Temporary runtime scratch workspace (`/temp/.keep`).

> [!NOTE]
> Do not use legacy paths like `/system/etc/` (use `/config/sys/`) or recreational directories (no `/apps/games/`).

---

## 5. Commit Conventions

Keira uses two consistent commit message formats:

### A. Release Commits:
```text
Keira <version>
```
*Example: `Keira 0.33.0`*

### B. Standard Conventional Commits:
```text
<type>(<scope>): <short imperative summary>

[optional detailed description explaining WHY, not just what]
```

#### Allowed Types:
* `feat`: New driver, syscall, command, or kernel capability.
* `fix`: Bug fix, panic resolution, or race condition mitigation.
* `refactor`: Structural reorganization without changing functionality.
* `perf`: Memory, scheduling, or I/O performance optimization.
* `docs`: Documentation addition, clarification, or link update.
* `style`: Formatting, comment syntax, or whitespace adjustments.
* `test`: Automated test harness additions or QEMU test improvements.
* `chore`: Build system, Makefile, or repository maintenance.
