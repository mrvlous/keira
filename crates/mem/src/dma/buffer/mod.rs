// SPDX-License-Identifier: GPL-2.0-only

//! Contiguous DMA memory buffer allocation and descriptor primitives.

pub mod allocator;

pub use allocator::{alloc_dma_buffer, DmaBuffer, ScatterGatherEntry};
