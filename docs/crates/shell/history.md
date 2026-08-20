<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Command History Ring Buffer

Documentation for history in [`crates/shell/src/history.rs`](../../../crates/shell/src/history.rs).

## Features
- Circular ring buffer storing recent shell commands.
- Up Arrow (`KEY_UP` `0x80`) / Down Arrow (`KEY_DOWN` `0x81`) navigation with dynamic line redraws.
