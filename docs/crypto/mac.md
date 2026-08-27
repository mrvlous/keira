<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Mandatory Access Control (MAC)

This document details rule-based Mandatory Access Control path enforcement in Keira Kernel.

---

## Security Policies

* **Root Lockdown**: Prevents direct writes to `/system/` by unprivileged tasks.
* **Device Access Rules**: Enforces fine-grained character and block device node permissions.

---

## Core API (`crates/task/src/security/mod.rs`)

```rust
pub fn mac_check_file_access(pid: u32, path: &str, is_write: bool) -> Result<(), &'static str>;
```
