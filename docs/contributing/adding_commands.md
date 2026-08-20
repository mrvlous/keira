<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Tutorial: Creating a Native Shell Command

Step-by-step guide for adding a new built-in command to `keira-shell`.

## Step 1: Create Command File
Create a new file under [`crates/shell/src/cmds/mycmd.rs`](../../crates/shell/src/cmds):
```rust
// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

use keira_io::vga;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    if let Some("-h") | Some("--help") = parts.next() {
        unsafe {
            vga::print_str("Usage: mycmd [options]\n");
        }
        return;
    }
    unsafe {
        vga::print_str("Hello from mycmd!\n");
    }
}
```

## Step 2: Register in `cmds/mod.rs` & `executor.rs`
1. Add `pub mod mycmd;` to `crates/shell/src/cmds/mod.rs`.
2. Add `"mycmd" => cmds::mycmd::run(parts)` inside `crates/shell/src/executor.rs`.
3. Add `"mycmd"` to `SHELL_CMDS` list in `Makefile`.
