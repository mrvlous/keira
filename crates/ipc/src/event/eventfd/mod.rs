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

//! Counter notification file descriptors (`eventfd`) and signal file descriptors (`signalfd`).

pub mod counter;
pub mod ops;

pub use counter::{
    close_eventfd, create_eventfd, get_eventfd_stats, get_eventfd_table, read_eventfd,
    write_eventfd, EventFd, EventFdEntry, EFD_CLOEXEC, EFD_NONBLOCK, EFD_SEMAPHORE, MAX_EVENTFDS,
};
pub use ops::{sys_eventfd, sys_signalfd};
