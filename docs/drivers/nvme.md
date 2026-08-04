# NVMe (Non-Volatile Memory Express) PCIe Controller Driver

This document details the 64-bit Memory-Mapped I/O (MMIO) register interface, Admin Queue pair initialization, I/O Queue creation, and NVMe Namespace management in Keira Kernel.

---

## 1. Subsystem Overview

Keira Kernel provides high-speed NVMe 1.4 PCIe SSD storage controller support ([nvme.rs](../../kernel/src/io/nvme.rs)), interfacing directly with PCIe ECAM config space and MMIO doorbells.

```
+-------------------------------------------------------------------------+
|                              VFS Layer                                  |
+-------------------------------------------------------------------------+
                                    |
                                    v
+-------------------------------------------------------------------------+
|                     NVMe PCIe Controller Driver                         |
|  +---------------------+  +--------------------+  +------------------+  |
|  | Admin SQ / CQ Ring  |  | Doorbell Registers |  | Namespace #1 Map |  |
|  +---------------------+  +--------------------+  +------------------+  |
+-------------------------------------------------------------------------+
                                    |
                                    v
+-------------------------------------------------------------------------+
|                  PCIe Express NVMe M.2 / U.2 SSD Hardware               |
+-------------------------------------------------------------------------+
```

---

## 2. Register Interface & Admin Queues

*   **CAP (Controller Capabilities)**: Offset `0x00` (Max Queue Entries, Doorbell Stride).
*   **CC (Controller Configuration)**: Offset `0x14` (Enable, I/O Completion Queue Entry Size `16`, I/O Submission Queue Entry Size `64`).
*   **AQA (Admin Queue Attributes)**: Offset `0x24` (Admin SQ / CQ Entry Depth).
*   **ASQ / ACQ**: Offset `0x28` / `0x30` (Physical frame addresses for Admin Submission and Completion queues).

---

## 3. Kernel APIs

*   `pub fn init(bus: u8, dev: u8, func: u8, mmio_base: u64) -> Result<(), &'static str>`: Configures PCIe MMIO space and initializes NVMe Admin Queues.
