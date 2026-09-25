<!-- SPDX-License-Identifier: GPL-2.0-only -->

# AHCI SATA Storage Controller Driver

The Advanced Host Controller Interface (AHCI) driver provides high-performance DMA access to SATA hard drives and solid-state drives.

---

## 1. AHCI Memory Map

AHCI controllers expose Memory-Mapped I/O via PCI BAR5 (`ABAR`):
* **Generic Host Control (GHC)**:
  * `0x00`: Host Capabilities (`CAP`).
  * `0x04`: Global Host Control (`GHC`, AHCI Enable bit 31).
  * `0x08`: Interrupt Status (`IS`).
  * `0x0C`: Ports Implemented (`PI`).
* **Port Registers (0x100 + port * 0x80)**:
  * `PxCLB`: Command List Base Address (1 KiB aligned).
  * `PxFB`: Received FIS Base Address (256-byte aligned).
  * `PxIS`: Port Interrupt Status.
  * `PxIE`: Port Interrupt Enable.
  * `PxCMD`: Port Command and Status (Start bit 0, FIS Receive Enable bit 4).
  * `PxTFD`: Task File Data (Status & Error registers).
  * `PxSSTS`: SATA Status (Device detection: `0x3` = Device present and PHY established).
  * `PxCI`: Command Issue bitmask (32 command slots).

---

## 2. DMA Command Execution Flow

1. Construct Command Header pointing to Command Table.
2. Fill Command Table with Command FIS (`0x27` Register H2D) requesting `READ DMA EXT` (`0x25`) or `WRITE DMA EXT` (`0x35`).
3. Set Physical Region Descriptor Table (PRDT) entries pointing to physical DMA destination frames.
4. Set slot bit in `PxCI`.
5. Await interrupt or poll `PxCI` bit until cleared by hardware DMA engine.
