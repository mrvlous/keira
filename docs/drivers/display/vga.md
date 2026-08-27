<!-- SPDX-License-Identifier: GPL-2.0-only -->

# VGA 80x25 Character Text Console Driver

This document details the hardware text mode console driver at physical address `0xB8000`.

---

## Technical Specifications

* **Buffer Address**: Physical `0xB8000` (mapped in kernel virtual memory).
* **Dimensions**: 80 columns $\times$ 25 rows = 2000 character cells (4000 bytes).
* **Hardware Cursor**: Programmed via CRTC ports `0x3D4` and `0x3D5`.

---

## Core API (`crates/io/src/vga/mod.rs`)

```rust
pub fn init();
pub fn set_color(fg: Color, bg: Color);
pub fn print_str(s: &str);
pub fn set_cursor_pos(row: u16, col: u16);
```
