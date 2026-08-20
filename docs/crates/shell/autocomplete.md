<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Tab Auto-Completion Engine

Documentation for auto-completion in [`crates/shell/src/autocomplete.rs`](../../../crates/shell/src/autocomplete.rs).

## Features
- Inspects typed command token on Tab key (`0x09`).
- Matches command prefixes against the 74 built-in commands.
- Traverses current directory in VFS to complete file and folder paths.
