<!--
SPDX-License-Identifier: GPL-2.0-only

Keira Kernel - Operating System Kernel
Copyright (C) 2026 Moh. Ananda Firmansyah Putra
-->

# Multi-Virtual Terminal (TTY) Subsystem

This document details virtual terminal switching, TTY device node routing, and console buffers in Keira Kernel.

---

## 1. Subsystem Overview

Keira Kernel provides a Multi-Virtual Terminal subsystem ([tty.rs](../../kernel/src/io/tty.rs)) allowing virtual console switching (`tty1` to `tty4`) via Alt+F1..F4 keyboard shortcuts.

---

## 2. Virtual Terminal Structure

```rust
pub struct VirtualTerminal {
    pub id: usize,
    pub buffer: [u16; 80 * 25],
    pub cursor_x: usize,
    pub cursor_y: usize,
}
```

---

## 3. Kernel APIs

*   `pub fn switch_tty(tty_id: usize)`: Swaps active VGA text buffer and cursor context.
