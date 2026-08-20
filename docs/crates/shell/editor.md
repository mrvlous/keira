<!-- SPDX-License-Identifier: GPL-2.0-only -->

# `kvi` Fullscreen Interactive Text Editor

Documentation for the text editor in [`crates/shell/src/editor/kvi.rs`](../../../crates/shell/src/editor/kvi.rs).

## Key Features
- **Visual Editing**: 24-line text buffer with dynamic line numbering and status bar.
- **Navigation**: Arrow keys (`Up`, `Down`, `Left`, `Right`), `Page Up`, `Page Down`, `Home`, `End`.
- **Search**: `F3` forward search across text buffer with cursor jump.
- **Save & Exit**: `Ctrl+S` / `F10` save confirmation dialog with path selection and `Ctrl+Q` exit safeguard.
