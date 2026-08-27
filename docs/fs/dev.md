<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Device Nodes & Virtual Device Filesystem (`/system/dev/`)

This document specifies the virtual device node manager exposing hardware and virtual streams through the standard VFS file interface.

---

## Standard Device Nodes

| Node Path | Device Type | Operations Supported | Description |
| :--- | :--- | :--- | :--- |
| `/system/dev/console` | Character | Read / Write | Interactive text VGA console and COM1 serial |
| `/system/dev/null` | Character | Read (EOF) / Write (Sink) | Discards all writes; returns 0 bytes on read |
| `/system/dev/zero` | Character | Read (Zeroes) / Write (Sink) | Streams infinite null bytes (`0x00`) |
| `/system/dev/random` | Character | Read (Entropy) | Hardware random entropy stream |
| `/system/dev/sda` | Block | Read / Write (Raw) | Raw primary physical storage drive |
| `/system/dev/sda1` | Block | Read / Write (Partition) | Primary FAT16 partition |

---

## Core API (`crates/fs/src/dev/mod.rs`)

```rust
pub fn read_dev_node(name: &str, buf: &mut [u8]) -> Result<usize, &'static str>;
pub fn write_dev_node(name: &str, buf: &[u8]) -> Result<usize, &'static str>;
```
