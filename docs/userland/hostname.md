<!-- SPDX-License-Identifier: GPL-2.0-only -->

# System Hostname Configuration

This document details hostname configuration and persistent network identity in Keira Kernel.

---

## 1. Subsystem Overview

Keira Kernel allows viewing and modifying the active system hostname (`hostname` command), persisted to `/system/etc/hostname` on storage disk.

---

## 2. Configuration API

```rust
pub static mut SYSTEM_HOSTNAME: [u8; 64] = *b"keira\0\0\0...";
```

---

## 3. Kernel APIs

*   `pub fn get_hostname() -> &'static str`: Returns active node hostname.
*   `pub fn set_hostname(name: &str) -> Result<(), &'static str>`: Updates hostname in RAM and disk file.
