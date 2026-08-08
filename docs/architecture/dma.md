<!--
SPDX-License-Identifier: GPL-2.0-only

Keira Kernel - Operating System Kernel
Copyright (C) 2026 Moh. Ananda Firmansyah Putra
-->

# Direct Memory Access (DMA) Scatter-Gather Allocator

This document details the physically contiguous DMA memory allocation, Scatter-Gather list mapping, and zero-copy bus master data transfer engine in Keira Kernel.

---

## 1. Subsystem Overview

High-speed hardware controllers (AHCI SATA, Intel e1000 NIC, Intel HDA Audio) require physically contiguous memory buffers for Direct Memory Access (DMA). Keira Kernel implements a dedicated DMA memory manager ([dma.rs](../../kernel/src/mem/dma.rs)).

---

## 2. Scatter-Gather Mapping

For data transfers exceeding single 4096-byte page boundaries, the allocator constructs Scatter-Gather Descriptor Lists (`ScatterGatherEntry`):

```rust
pub struct ScatterGatherEntry {
    pub phys_addr: u64, // Physical frame base address
    pub length: u32,    // Segment byte length
}
```

---

## 3. Kernel APIs

*   `pub fn alloc_dma_buffer(size: usize) -> Result<DmaBuffer, &'static str>`: Pops physically contiguous page frames from the PMM and maps them for hardware bus master DMA transfers.
