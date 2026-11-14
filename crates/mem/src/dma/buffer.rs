// SPDX-License-Identifier: GPL-2.0-only
/*
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra <mrvlous@proton.me>
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 */

//! DMA memory buffer allocator and scatter-gather lists.

pub mod allocator {
    use crate::pmm;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct ScatterGatherEntry {
        pub phys_addr: u64,
        pub length: u32,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct DmaBuffer {
        pub vaddr: u64,
        pub paddr: u64,
        pub size: usize,
    }

    pub fn alloc_dma_buffer(size: usize) -> Result<DmaBuffer, &'static str> {
        let frame = pmm::alloc_frame().ok_or("Out of physical frames for DMA buffer")?;
        Ok(DmaBuffer {
            vaddr: frame,
            paddr: frame,
            size,
        })
    }
}

pub use self::allocator::*;
