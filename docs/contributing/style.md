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

### Source Files (`.rs`, `.c`, `.h`), `Makefile`, & `Cargo.toml`
Must include the standard 1-paragraph GPL-2.0-only license block:
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

### Configuration Files (`.gitignore`, `rust-toolchain.toml`, `rustfmt.toml`)
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
- **No Decorative Symbols**: Never use decorative divider lines such as `---` or `===` within comments.
- **Grammar**: Use formal, technical English. Zero non-English code comments permitted.
