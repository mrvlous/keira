<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Coding Style & Documentation Standards

Standards for writing clean, maintainable, and safe kernel code.

## License Header Policy
Every source file (`.rs`, `.c`, `.h`, `.asm`), build script, and markdown file MUST begin with a license header.
**Do NOT include author email addresses in file headers.**

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

For Markdown files:
```markdown
<!-- SPDX-License-Identifier: GPL-2.0-only -->
```

## Rust Conventions
- Use `snake_case` for modules, functions, and variables.
- Use `UpperCamelCase` for structs, enums, traits, and type aliases.
- Use `SCREAMING_SNAKE_CASE` for global constants and static variables.
- Write formal, clear English rustdoc comments (`///`) on all public types and functions.
- Run `cargo fmt --all` before committing.

## C Conventions
- Follow Linux kernel C style guidelines (4-space indentation, braces on new line for functions).
- Run `make format` to run `clang-format` on all C and header files.
