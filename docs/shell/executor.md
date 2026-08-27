<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Shell Command Executor & Parser

This document details string tokenization, variable substitution, and command routing in Keira Kernel.

---

## Tokenization Pipeline

1. **Whitespace Trimming**: Strips leading and trailing space characters.
2. **Variable Expansion**: Replaces `$USER`, `$PATH`, `$HOME`, `$HOSTNAME` with active environment string values.
3. **Dispatching**: Matches first token against registered command handlers or routes to userland execution (`run`).

---

## Core API (`crates/shell/src/executor.rs`)

```rust
pub fn execute_command(input: &str);
pub fn parse_arguments<'a>(input: &'a str, argv: &mut [&'a str; 16]) -> usize;
```
