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

//! Ring buffer pipe primitives and unidirectional FIFO channels.

pub mod buffer;
pub mod ops;

pub use buffer::{
    PipeBuffer, PipeChannel, MAX_PIPES, O_CLOEXEC, O_NONBLOCK, PIPE_BUFFER_CAPACITY,
    PIPE_BUFFER_SIZE, PIPE_TABLE, SYSTEM_PIPE,
};
pub use ops::{
    close_pipe_read, close_pipe_write, create_pipe, create_pipe_channel, dup_pipe_endpoint,
    get_pipe_stats, get_pipe_table, pipe_table_reset, query_pipe_readiness, read_pipe,
    read_pipe_channel, write_pipe, write_pipe_channel,
};
