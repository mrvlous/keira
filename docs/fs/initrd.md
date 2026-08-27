<!-- SPDX-License-Identifier: GPL-2.0-only -->

# USTAR Boot RAM Disk Archive (`initrd`)

This document specifies the in-memory USTAR archive reader loaded by GRUB during system bootstrap.

---

## USTAR Header Format (512 Bytes)

```
0        100 108     116     124     136     148     156 157      257
+-----------+-------+-------+-------+-------+-------+---+--------+
| File Name | Mode  | UID   | GID   | Size  | MTime |Chk|TypeFlag|
| (100 B)   | (8 B) | (8 B) | (8 B) | (12 B)| (12 B)|(8)| (1 B)  |
+-----------+-------+-------+-------+-------+-------+---+--------+
```

---

## Core API (`crates/fs/src/tar/mod.rs`)

```rust
pub fn initrd_init(start_addr: usize, end_addr: usize);
pub fn list_initrd_files();
pub fn exists(target: &str) -> bool;
pub fn read_file_content(target: &str, buf: &mut [u8]) -> Result<usize, &'static str>;
```
