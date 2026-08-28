<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Interactive Tab Autocompletion Engine

This document specifies the interactive command and file path autocompletion engine in `keira-shell`.

---

## Autocompletion Pipeline

```mermaid
graph TD
    TabKey["User Presses 'Tab' Key"] --> Lexer["Parse Current Input Line into Tokens"]
    Lexer --> CheckPosition{"Cursor on First Word (Command) or Argument (Path)?"}
    CheckPosition -->|Command| ScanManifest["Scan SHELL_CMDS Registry Table"]
    CheckPosition -->|Path Argument| ScanVFS["Enumerate Active Directory Entries via FAT16/VFS"]
    ScanManifest --> MatchPrefix["Find Common Prefix Matches"]
    ScanVFS --> MatchPrefix
    MatchPrefix --> Action{"Match Count"}
    Action -->|Single Match| CompleteWord["Inline Auto-Complete Token + Append Space/Slash"]
    Action -->|Multiple Matches| DisplaySuggestions["Render Formatted Grid of Possible Completions"]
```

---

## Technical Specifications

| Parameter | Specification | Description |
| :--- | :--- | :--- |
| **Trigger Key** | ASCII Tab (`0x09`) | Standard interactive trigger |
| **Match Domains** | Commands, Executables (`.elf`), File Paths, Directories | Context-aware suggestion scoping |
| **Directory Trailing** | Appends trailing slash `/` | Allows fluid traversal through subdirectories |

---

## Core API (`crates/shell/src/autocomplete.rs`)

```rust
/// Compute auto-completion suggestions for current command line buffer.
pub fn handle_tab_completion(line_buf: &mut [u8], len: &mut usize, cursor_pos: &mut usize);
```
