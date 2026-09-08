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
/// Initialize default hardware or synthetic NVMe PCIe controller instance.
pub fn ensure_initialized();

/// Initialize NVMe PCIe controller and Admin Queue pairs.
pub fn init(bus: u8, dev: u8, func: u8, mmio_base: u64) -> Result<(), &'static str>;

/// Retrieve immutable copy of current NVMe controller state.
pub fn get_nvme_controller() -> Option<NvmeController>;

/// Retrieve NVMe operational stats: (ready, active_namespaces, total_capacity_mb).
pub fn get_nvme_stats() -> (bool, u32, u64);
```
