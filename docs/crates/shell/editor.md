<!-- SPDX-License-Identifier: GPL-2.0-only -->

# GNU nano-Style Interactive Text Editor (`edit` / `nano`)

Documentation for the fullscreen terminal text editor in [`crates/shell/src/editor/kvi.rs`](../../../crates/shell/src/editor/kvi.rs).

## Key Features
- **Authentic GNU nano Aesthetic**: Fullscreen 80x25 VGA canvas with top title bar, line numbering gutter (`   1 | `), message/status bar, and 2-row shortcut key matrix.
- **Syntax Highlighting**: Real-time syntax highlighting for Rust/C keywords, strings, comments, numbers, and operators conforming to the standard Linux console palette.
- **Position Telemetry (`^C`)**: Detailed line, column, and character byte counts with progress percentages.
- **Clipboard Management (`^K` / `^U`)**: In-memory line cut and paste operations.
- **Interactive Search (`^W`)**: Text search across buffer with dynamic match highlighting and automatic viewport scrolling.
- **Persistent Storage (`^O`)**: Direct FAT16 filesystem writeout with modified dirty buffer tracking and `^X` exit confirmation safeguarding.
- **Command Aliases**: Invokable via both `edit <file>` and `nano <file>`.
