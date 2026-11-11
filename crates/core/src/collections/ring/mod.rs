// SPDX-License-Identifier: GPL-2.0-only

//! Circular ring buffer FIFO queue subsystem.
//!
//! Provides static bounded queues and circular byte/event buffers for kernel paths.

pub mod buffer;

pub use buffer::RingBuffer;
