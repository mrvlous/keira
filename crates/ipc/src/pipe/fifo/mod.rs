// SPDX-License-Identifier: GPL-2.0-only

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
