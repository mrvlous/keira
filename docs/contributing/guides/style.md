<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Coding Style & Documentation Standards

This document establishes the official coding, documentation, licensing, and commit conventions for Keira Kernel.

---

## 1. License Header Policy & SPDX Architecture

Every source file (`.rs`, `.c`, `.h`, `.asm`, `.inc`), build script (`Makefile`), configuration file (`.toml`, `.json`), and documentation file (`.md`) **MUST** begin with a standard machine-readable SPDX license identifier mapping to the canonical texts in [`LICENSES/`](../../../LICENSES/README.md).

> [!IMPORTANT]
> **License Header Rules:**
> 1. **Canonical Texts**: Full legal texts for all supported SPDX identifiers are maintained in [`LICENSES/preferred/`](../../../LICENSES/preferred/) and [`LICENSES/exceptions/`](../../../LICENSES/exceptions/).
> 2. **Name Only (No Email Addresses)**: Always use the author's full name without email addresses (e.g. `Copyright (C) 2026 Moh. Ananda Firmansyah Putra`). Email addresses are maintained exclusively in `MAINTAINERS` and `CREDITS`.
> 3. **Original Author vs. Contributors**:
>    - For files authored by the primary creator, use `Copyright (C) 2026 Moh. Ananda Firmansyah Putra`.
>    - When an external contributor authors a new standalone file, they must use their own full name: `Copyright (C) 2026 <Contributor Full Name>`.
>    - When significantly modifying existing files, contributors may append an additional copyright line below existing authors:
>      ```text
>      Copyright (C) 2026 Moh. Ananda Firmansyah Putra
>      Copyright (C) 2026 <Contributor Full Name>
>      ```
>    - Contributors should also append their name and contact information to the root [`CREDITS`](../../../CREDITS) file.

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
*Example: `Keira 0.36.0`*

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

---

## 6. Console Output & CLI Styling Standards

All shell commands, driver logging, and terminal output must strictly adhere to the **Standard 3 Monochrome Linux Console Palette**:

### A. Color Palette Matrix:
| Element / Role | Color Constant | Hex RGB | Semantic Usage |
| :--- | :--- | :--- | :--- |
| **Headers & Prompts** | `vga::Color::White` | `#FFFFFF` | Table column headers, command prompts (`admin@keira:~$`), titles, category headings. |
| **Body & Data Lines** | `vga::Color::LightGrey` | `#AAAAAA` | Default stdout, file listings, telemetry metrics, register values, memory addresses. |
| **Success Badges** | `vga::Color::LightGreen` | `#55FF55` | Minimal status tokens: `[OK]`, `[Mounted]`, `RUNNING`, `UP (e1000)`, `Connecting`, `Downloading`, `Finished`. |
| **Warning Badges** | `vga::Color::Yellow` | `#FFFF55` | Alerts, cache invalidations, fallback notices: `[ WARN ]`, `[REBUILDING]`, `Warning`. |
| **Error Messages** | `vga::Color::LightRed` | `#FF5555` | Critical failures, permission errors: `[FAILED]`, `Error`, `error`. |

> [!CAUTION]
> **Strict Prohibition**: Never use recreational or non-standard console colors (`Cyan`, `LightCyan`, `Magenta`, `LightBlue`, `Brown`). The console palette must remain austere, professional, and consistent with the Linux monochrome standard.

### B. CLI Argument & Flag Parser Conventions:
- Shell commands with flags must utilize the `#![no_std]` [`CliArgs`](file:///crates/shell/src/args.rs) parser engine.
- Support standard POSIX single-letter short flags (`-l`, `-a`, `-c`, `-m`, `-s`, `-v`, `-u`, `-f`, `-r`, `-d`, `-t`, `-n`, `-L`) and GNU long flags (`--long`, `--all`, `--version`, `--help`).
- Commands without arguments or configurations (`sync`, `reset`, `unwind`, `runtime`, `wipe`) execute immediately without blocking on `-h` boilerplate.
- Network download and streaming progress bars must adhere to the `rustc`/`cargo` compiler format with 12-character right-aligned status tags (`Connecting`, `Downloading`, `Downloaded`, `Finished`) and size metrics (`Bytes`, `KiB`, `MiB`).

---

## 7. Whitespace, Newline & Formatting Standards

1. **Single Trailing Newline**:
   - Every file (`.rs`, `.c`, `.h`, `.asm`, `.inc`, `.md`, `.toml`, `.json`, `.ld`, `Makefile`) **MUST** end with exactly one newline (`\n`).
   - No missing trailing newline and no multiple trailing newlines at the end of files.
2. **Consecutive Blank Lines Policy**:
   - Multiple consecutive blank lines (`\n\n\n+`) are strictly prohibited across all source code, documentation, and build scripts.
   - Use at most one blank line between function declarations, struct definitions, and markdown paragraphs.
3. **No Trailing Whitespace**:
   - Lines must never contain trailing whitespace characters (`\s+$`).
   - Enforced automatically via `cargo fmt`, `clang-format`, and `make format`.

---

## 8. Language & Grammar Standards

1. **Strict 100% English Policy**:
   - All code comments, docstrings (`//!`, `///`), documentation files, commit messages, and terminal outputs **MUST** be written in formal, grammatically correct English.
   - Non-English comments or phrases (e.g. Indonesian) are strictly forbidden in the codebase.
2. **Grammar & Tone**:
   - Write clear, concise, and professional documentation and docstrings.
   - Use imperative mood for commit summaries (e.g. `"feat(user): add dual-architecture support..."`) and spell out words like `"and"` instead of ampersands (`&`) in commit subjects.
3. **Comment Formatting & Decorative Banner Policy**:
   - Decorative banner symbols in code comments (such as `// ===`, `// ---`, `/* === */`, or `; ===`) are strictly prohibited.
   - Use clean, concise single-line `//` comments or formal rustdoc docstrings (`//!`, `///`) without ASCII art or divider lines.

---

## 9. Multi-Architecture Target Organization

1. **Target Specification Layout**:
   - Architecture JSON specification files must be organized in architecture-specific subdirectories under `targets/`:
     - `targets/x86/x86_64-keira-none.json` (64-bit Long Mode)
     - `targets/x86/i686-keira-none.json` (32-bit Protected Mode)
2. **Linker Scripts**:
   - Kernel Linkers: `arch/x86/linker.ld` (`x86_64`) and `arch/x86/linker32.ld` (`i686`).
   - Userland Linkers: `user/arch/x86/linker.ld` (`x86_64` base `0x40000000`) and `user/arch/x86/linker32.ld` (`i686` base `0x01000000`).
