<!-- SPDX-License-Identifier: GPL-2.0-only -->

# DMA Buffer Allocation & Scatter-Gather

This document specifies physically contiguous DMA buffers used by PCI bus master devices (AHCI SATA, Intel e1000, NVMe) in Keira Kernel.

---

## DMA Buffer Structure

```rust
pub struct DmaBuffer {
    pub virt_addr: usize,
    pub phys_addr: usize,
    pub size_bytes: usize,
}

pub struct ScatterGatherEntry {
    pub phys_addr: u64,
    pub byte_count: u32,
}
```

---

## API & Allocation

```rust
/// Allocate a physically contiguous buffer aligned to a 4KB boundary.
pub fn alloc_dma_buffer(size: usize) -> Option<DmaBuffer>;

/// Free a DMA buffer back to physical frame allocator.
pub fn free_dma_buffer(buf: DmaBuffer);
```
