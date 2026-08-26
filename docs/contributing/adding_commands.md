<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Tutorial: Creating a Native Shell Command

Step-by-step guide for adding a new built-in command to `keira-shell`.

## Step 1: Create Command File
Select the appropriate category subfolder in `crates/shell/src/cmds/` (`fs`, `sys`, `proc`, `net`, `sec`, `dev`, or `util`), and create a new file under [`crates/shell/src/cmds/<category>/mycmd.rs`](../../crates/shell/src/cmds):
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

## Step 2: Register in `cmds/<category>/mod.rs` & `executor.rs`
1. Add `pub mod mycmd;` to `crates/shell/src/cmds/<category>/mod.rs`.
2. Add `"mycmd" => super::cmds::mycmd::run(&mut parts),` inside `crates/shell/src/executor.rs`.
3. Add `mycmd` to the `SHELL_CMDS` manifest in [`Makefile`](../../Makefile).
