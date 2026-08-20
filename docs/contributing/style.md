<!--
SPDX-License-Identifier: GPL-2.0-only

Keira Kernel - Operating System Kernel
Copyright (C) 2026 Moh. Ananda Firmansyah Putra
-->

# Coding, Comment, and License Style Guidelines

To maintain code quality, security, and maintainability across the **Keira Kernel** codebase, all contributions must adhere to the style and licensing standards outlined below.

## 1. Rust and C Formatting Rules

- **Rust Edition**: Rust 2021 (no_std, freestanding).
- **Line Length**: 100 characters maximum (enforced via `rustfmt.toml`).
- **Indentation**: 4 spaces, no hard tabs.
- **C Block Comments**: Single-line C++ style comments (`//`) in C source files are strictly forbidden. Use C-style block comments (`/* ... */`).

## 2. License Comment Header Rules

Every file in the repository must begin with the appropriate standardized GPL-2.0-only license header:

### Source Files (`.rs`, `.c`, `.h`, `.asm`, `.inc`, `.ld`), `Makefile`, & `Cargo.toml`
Must include the standard 1-paragraph GPL-2.0-only license block adapted to the file comment syntax:

* **Rust Files (`.rs`)**:
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

* **C and Linker Files (`.c`, `.h`, `.ld`)**:
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

* **Assembly Files (`.asm`, `.inc`)**:
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

* **Build & Package Configurations (`Makefile`, `Cargo.toml`)**:
```toml
# SPDX-License-Identifier: GPL-2.0-only
#
# Keira Kernel - Operating System Kernel
# Copyright (C) 2026 Moh. Ananda Firmansyah Putra
#
# This program is free software; you can redistribute it and/or modify
# it under the terms of the GNU General Public License as published by
# the Free Software Foundation; version 2 of the License.
```

### Configuration Files (`.gitignore`, `rust-toolchain.toml`, `rustfmt.toml`, `.clang-format`)
Must include the 4-line concise copyright comment block:
```toml
# SPDX-License-Identifier: GPL-2.0-only
#
# Keira Kernel - Operating System Kernel
# Copyright (C) 2026 Moh. Ananda Firmansyah Putra
```

### Markdown Documentation Files (`docs/**/*.md`)
Must include the HTML comment block:
```markdown
<!--
SPDX-License-Identifier: GPL-2.0-only

Keira Kernel - Operating System Kernel
Copyright (C) 2026 Moh. Ananda Firmansyah Putra
-->
```

*(Note: Root `README` and auto-generated `Cargo.lock` are kept clean without header blocks).*

### `LICENSE` File
Contains the full GNU General Public License v2.0 text.

## 3. Documentation Style Rules

- **Module-Level Rustdocs (`//!`)**: Required at top of every Rust source file after the license header.
- **Item Rustdocs (`///`)**: Required for all public structs, enums, fields, and functions.
- **No Decorative Symbols**: Never use decorative divider lines such as `---` or `===` within code comments.
- **Grammar**: Use formal, technical English. Zero non-English code comments permitted.

## 4. Git Commit & Release Tagging Conventions

- **Release Commits**: Standard release commit message format is `Keira <version>` (e.g. `Keira 0.28.6`).
- **Release Tags**: Annotated git tag format is `v<version>` with message `Keira <version>` (e.g. `git tag -a v0.28.6 -m "Keira 0.28.6"`).
- **Patch Bumps**: When advancing versions, update both `Cargo.toml` and `Cargo.lock` synchronously.
