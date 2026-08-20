<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Kernel Panic Handler & Blue Screen of Death

Documentation for panic handling in [`crates/kernel/src/panic.rs`](../../../crates/kernel/src/panic.rs).

## Features
- **Serial Output**: Emits fatal panic messages, source file, line number, and stack location to COM1 serial port.
- **Visual Display**: Clears VGA text mode console to solid Blue with high-visibility White panic diagnostics.
- **CPU Halting**: Disables hardware interrupts (`cli`) and enters an infinite `hlt` loop to prevent memory corruption or hardware damage.
