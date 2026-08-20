<!-- SPDX-License-Identifier: GPL-2.0-only -->

# VGA Text Mode Console Driver

Documentation for the 80x25 text console driver in [`crates/io/src/vga/`](../../../crates/io/src/vga).

## Architecture
- Text buffer mapped at physical base `0xB8000`.
- Supports 16 foreground and 16 background colors.
- Provides scrolling, backspace handling, formatted numeric printing (`print_u64`, `print_hex`), and startup boot logging tags `[ OK ]`, `[WARN]`, `[FAIL]`.
