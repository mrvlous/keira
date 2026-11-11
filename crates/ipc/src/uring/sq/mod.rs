// SPDX-License-Identifier: GPL-2.0-only

//! io_uring Submission Queue descriptors and memory layouts.

pub mod entry;
pub mod ring;

pub use entry::{
    SubmissionQueueEntry, IOSQE_ASYNC, IOSQE_BUFFER_SELECT, IOSQE_FIXED_FILE, IOSQE_IO_DRAIN,
    IOSQE_IO_HARDLINK, IOSQE_IO_LINK,
};
pub use ring::IoSqringOffsets;
