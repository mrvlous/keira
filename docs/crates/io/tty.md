<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Multi-Virtual Terminal Subsystem (`tty1` - `tty4`)

Documentation for virtual terminals in [`crates/io/src/tty/`](../../../crates/io/src/tty).

## Features
- Manages 4 independent virtual text consoles (`tty1` to `tty4`).
- Supports fast console switching via Alt+F1..F4 keyboard shortcuts.
- Isolates cursor state, color attributes, screen buffers, and input queues per virtual terminal.
