// SPDX-License-Identifier: GPL-2.0-only

//! Contiguous physical Direct Memory Access (DMA) buffer allocation.
//!
//! Subdivided into specialized buffer allocation implementations and scatter-gather representations.

pub mod buffer;

pub use buffer::{alloc_dma_buffer, DmaBuffer, ScatterGatherEntry};

#[cfg(test)]
mod tests;
