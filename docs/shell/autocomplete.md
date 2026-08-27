<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Tab Auto-Completion Engine

This document specifies the auto-completion engine supporting command names, file paths, and device nodes.

---

## Completion Algorithm

1. **Token Extraction**: Identifies the trailing word fragment under the cursor.
2. **Catalog Matching**: Queries built-in command names, active working directory file entries, and `/system/dev/` device nodes.
3. **Common Prefix Expansion**: Expands the input buffer to the longest unambiguous common prefix.

---

## Core API (`crates/shell/src/autocomplete.rs`)

```rust
pub fn handle_tab_completion();
```
