<!-- SPDX-License-Identifier: GPL-2.0-only -->

# TTY Line Discipline

This document details canonical line buffering, raw character pass-through, and ANSI escape code processing.

---

## Operating Modes

* **Canonical Mode (`ICANON`)**: Input is accumulated in a line buffer until `\n` is encountered, supporting `Backspace` (`0x08`) editing.
* **Raw Mode**: Characters are returned immediately without line buffering or echo.

---

## Core API (`crates/io/src/tty/ldisc.rs`)

```rust
pub fn process_input_byte(c: u8) -> Option<u8>;
pub fn set_echo(enabled: bool);
pub fn set_canonical(enabled: bool);
```
