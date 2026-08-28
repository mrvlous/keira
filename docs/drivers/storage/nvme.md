<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Non-Volatile Memory Express (NVMe) Driver

This document specifies the NVMe over PCIe storage controller driver, circular Submission/Completion Queues (SQ/CQ), and Doorbell register management in Keira Kernel.

---

## NVMe Queue Architecture

```mermaid
graph TD
    subgraph Submission Queue (SQ)
        SQ0["SQ Entry 0 (64-byte Read Command)"] --> SQ1["SQ Entry 1 (64-byte Write Command)"]
        SQ1 --> SQ_N["..."]
    end
    subgraph Completion Queue (CQ)
        CQ0["CQ Entry 0 (16-byte Status/Phase)"] --> CQ1["CQ Entry 1 (16-byte Status/Phase)"]
        CQ1 --> CQ_N["..."]
    end
    SQ0 --> Doorbell["Ring SQ Tail Doorbell (MMIO Offset)"]
    Doorbell --> Controller["NVMe PCIe Controller Hardware"]
    Controller --> CQ0
```

---

## Technical Specifications

| Parameter | Specification | Description |
| :--- | :--- | :--- |
| **PCI Class** | `0x010802` | Non-Volatile Memory Controller |
| **Queue Protocol** | 64-byte SQ Entries, 16-byte CQ Entries | High-performance lock-free circular queues |
| **Admin Queues** | 1 Admin SQ / 1 Admin CQ | Controller configuration and namespace identification |
| **I/O Queues** | Dedicated I/O SQ / CQ pairs | Asynchronous block read/write operations |

---

## Core API (`crates/io/src/storage/nvme.rs`)

```rust
/// Initialize NVMe controller, map MMIO registers, and create I/O queue pairs.
pub unsafe fn init() -> Result<(), &'static str>;

/// Read blocks from NVMe namespace using asynchronous submission queue commands.
pub unsafe fn read_blocks(ns_id: u32, lba: u64, block_count: u16, buf: &mut [u8]) -> Result<(), &'static str>;

/// Write blocks to NVMe namespace.
pub unsafe fn write_blocks(ns_id: u32, lba: u64, block_count: u16, buf: &[u8]) -> Result<(), &'static str>;
```
