<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Multi-Virtual Terminals (`tty1`–`tty4`)

This document specifies virtual terminal context multiplexing and keyboard switching in Keira Kernel.

---

## Technical Specifications

* **Number of Virtual Consoles**: 4 independent TTY consoles (`tty1` through `tty4`).
* **Console Buffer**: Dedicated 80 $\times$ 25 character and color buffer per virtual terminal.
* **Hotkey Switching**: `Alt+F1` (`tty1`), `Alt+F2` (`tty2`), `Alt+F3` (`tty3`), `Alt+F4` (`tty4`).

---

## Core API (`crates/io/src/tty/mod.rs`)

```rust
pub fn switch_tty(index: usize);
pub fn get_active_tty() -> usize;
pub fn tty_write(tty_idx: usize, data: &[u8]);
```
