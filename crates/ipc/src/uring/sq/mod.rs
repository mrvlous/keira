// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! io_uring Submission Queue descriptors and memory layouts.

pub mod entry;
pub mod ring;

pub use entry::{
    SubmissionQueueEntry, IOSQE_ASYNC, IOSQE_BUFFER_SELECT, IOSQE_FIXED_FILE, IOSQE_IO_DRAIN,
    IOSQE_IO_HARDLINK, IOSQE_IO_LINK,
};
pub use ring::IoSqringOffsets;
