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

//! io_uring Submission Queue descriptors and memory layouts.

pub mod entry;
pub mod ring;

pub use entry::{
    SubmissionQueueEntry, IOSQE_ASYNC, IOSQE_BUFFER_SELECT, IOSQE_FIXED_FILE, IOSQE_IO_DRAIN,
    IOSQE_IO_HARDLINK, IOSQE_IO_LINK,
};
pub use ring::IoSqringOffsets;
