// SPDX-License-Identifier: GPL-2.0-only

//! High-performance asynchronous I/O engine (`io_uring`).

pub mod cq;
pub mod engine;
pub mod sq;

pub use cq::{CompletionQueueEntry, IoCqringOffsets};
pub use engine::*;
pub use sq::{
    IoSqringOffsets, SubmissionQueueEntry, IOSQE_ASYNC, IOSQE_BUFFER_SELECT, IOSQE_FIXED_FILE,
    IOSQE_IO_DRAIN, IOSQE_IO_HARDLINK, IOSQE_IO_LINK,
};

pub mod queue {
    pub use super::cq::{CompletionQueueEntry, IoCqringOffsets};
    pub use super::engine::*;
    pub use super::sq::{
        IoSqringOffsets, SubmissionQueueEntry, IOSQE_ASYNC, IOSQE_BUFFER_SELECT, IOSQE_FIXED_FILE,
        IOSQE_IO_DRAIN, IOSQE_IO_HARDLINK, IOSQE_IO_LINK,
    };
}

#[cfg(test)]
mod tests;
