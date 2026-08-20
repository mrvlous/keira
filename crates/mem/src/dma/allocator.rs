// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Contiguous physical Direct Memory Access (DMA) buffer allocation.

use crate::pmm;

/// A Scatter-Gather list entry mapping a physical address with length.
pub struct ScatterGatherEntry {
    pub phys_addr: u64,
    pub length: u32,
}

/// A contiguous physical memory DMA buffer descriptor.
pub struct DmaBuffer {
    pub vaddr: u64,
    pub paddr: u64,
    pub size: usize,
}

/// Allocate a physically contiguous DMA memory buffer for hardware bus master drivers.
pub fn alloc_dma_buffer(size: usize) -> Result<DmaBuffer, &'static str> {
    let frame = pmm::alloc_frame().ok_or("Out of physical frames for DMA buffer")?;
    Ok(DmaBuffer {
        vaddr: frame,
        paddr: frame,
        size,
    })
}
