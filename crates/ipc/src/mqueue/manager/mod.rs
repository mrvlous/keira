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

//! Message queue registry management and POSIX syscall handlers.

pub mod ops;
pub mod table;

pub use ops::sys_mq_open;
pub use table::{
    find_queue_mut, get_mqueue_stats, get_mqueue_table, mq_open, mq_receive, mq_send, mq_unlink,
    mq_unlink_by_id, parse_u32, MQUEUE_TABLE,
};
