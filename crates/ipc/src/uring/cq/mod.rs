// SPDX-License-Identifier: GPL-2.0-only

//! io_uring Completion Queue descriptors and memory layouts.

pub mod entry;
pub mod ring;

pub use entry::CompletionQueueEntry;
pub use ring::IoCqringOffsets;
