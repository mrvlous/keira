<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Command History Ring Buffer

This document details the circular history ring buffer supporting Up and Down arrow key history navigation.

---

## Technical Specifications

* **History Slots**: 16 circular buffer slots.
* **Buffer Capacity**: 256 bytes per history entry.
* **Navigation Index**: Tracks current browsing offset.

---

## Core API (`crates/shell/src/history.rs`)

```rust
pub fn history_add(command: &str);
pub fn history_prev();
pub fn history_next();
```
