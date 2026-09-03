<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Shell Command Dispatcher & Execution Pipeline

This document specifies the command parser, argument tokenizer, redirection router, and execution pipeline in `keira-shell`.

---

## Command Execution Architecture

```mermaid
graph TD
    Input["Raw Shell Input String"] --> SplitWhitespace["Whitespace Tokenizer (CliArgs / SplitWhitespace)"]
    SplitWhitespace --> BuiltinCheck{"Built-in Shell Command?"}
    BuiltinCheck -->|Yes| BuiltinDispatch["Dispatch to Category Handler (sys, fs, net, proc, etc.)"]
    BuiltinCheck -->|No| ELFCheck{"Executable File Exists on Disk (.elf)?"}
    ELFCheck -->|Yes| SpawnUserTask["VMM Address Space Creation & Ring 3 ELF Execution"]
    ELFCheck -->|No| UnknownCmd["Print 'Command not found' Error"]
```

---

## Pipelines & I/O Redirection

The shell executor natively parses UNIX-style pipeline and file redirection operators:

1. **Pipeline (`|`)**: Connects stdout of the left command into the in-memory `PIPE_BUFFER`, making it available for subsequent stream consumers (e.g. `tasks | search kernel`, `list | search .elf`).
2. **File Output Redirection (`>`)**: Intercepts console output and writes the stream to a specified file path in FAT16/VFS (e.g. `system > /temp/sysinfo.txt`).
3. **File Append Redirection (`>>`)**: Appends console output stream to the end of an existing file.
4. **Input Redirection (`<`)**: Reads data from a file into the active shell pipe stream.

---

## Technical Specifications

| Parameter | Specification | Description |
| :--- | :--- | :--- |
| **Line Buffer Capacity** | 256 bytes | Maximum length of single interactive command |
| **Max Arguments** | 16 positional arguments & flags | Statically bounded argument storage |
| **Pipe Buffer Capacity** | 4096 bytes | Zero-copy circular memory buffer for stream operations |
| **Privilege Escalation** | `please <command>` / `sudo` | Elevates administrative privileges for sensitive operations |

---

## Core API (`crates/shell/src/executor.rs`)

```rust
/// Parse and execute an input command line string.
pub fn execute_command(line: &str);

/// Check if active shell session possesses administrative privileges.
pub unsafe fn is_admin_mode() -> bool;
```
