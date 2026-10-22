// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Dynamic anonymous pipe read, write, channel management and descriptor lifecycle.

#![allow(static_mut_refs)]

use crate::pipe::fifo::buffer::{
    PipeChannel, MAX_PIPES, NEXT_PIPE_ID, O_NONBLOCK, PIPE_BUFFER_CAPACITY, PIPE_TABLE, SYSTEM_PIPE,
};

/// Write data bytes into the legacy kernel IPC pipe buffer.
///
/// # Safety
/// Caller must ensure single-threaded kernel execution or cooperative task context.
pub unsafe fn write_pipe(buf: &[u8]) -> usize {
    let pipe_ptr = &raw mut SYSTEM_PIPE;
    let pipe = &mut *pipe_ptr;
    let mut written = 0usize;

    for &b in buf {
        if pipe.count >= 1024 {
            break;
        }
        pipe.data[pipe.write_pos] = b;
        pipe.write_pos = (pipe.write_pos + 1) % 1024;
        pipe.count += 1;
        written += 1;
    }

    written
}

/// Read available data bytes from the legacy kernel IPC pipe buffer.
///
/// # Safety
/// Caller must ensure single-threaded kernel execution or cooperative task context.
pub unsafe fn read_pipe(buf: &mut [u8]) -> usize {
    let pipe_ptr = &raw mut SYSTEM_PIPE;
    let pipe = &mut *pipe_ptr;
    let mut read_bytes = 0usize;

    for slot in buf.iter_mut() {
        if pipe.count == 0 {
            break;
        }
        *slot = pipe.data[pipe.read_pos];
        pipe.read_pos = (pipe.read_pos + 1) % 1024;
        pipe.count -= 1;
        read_bytes += 1;
    }

    read_bytes
}

/// Create a legacy pipe descriptor pair (sys_pipe Vector 23).
///
/// # Safety
/// Resets global pipe state and returns descriptor pair (read_fd, write_fd).
pub unsafe fn create_pipe() -> Result<(usize, usize), &'static str> {
    let pipe_ptr = &raw mut SYSTEM_PIPE;
    let pipe = &mut *pipe_ptr;
    pipe.read_pos = 0;
    pipe.write_pos = 0;
    pipe.count = 0;
    Ok((3, 4))
}

/// Allocate a new in-kernel anonymous pipe channel with specified creation flags.
///
/// # Safety
/// Directly mutates static `PIPE_TABLE` and `NEXT_PIPE_ID`.
pub unsafe fn create_pipe_channel(flags: u32) -> Result<u32, &'static str> {
    for slot in PIPE_TABLE.iter_mut() {
        if !slot.in_use {
            let id = NEXT_PIPE_ID;
            NEXT_PIPE_ID = NEXT_PIPE_ID.wrapping_add(1);
            if NEXT_PIPE_ID == 0 {
                NEXT_PIPE_ID = 1;
            }

            *slot = PipeChannel {
                id,
                in_use: true,
                data: [0u8; PIPE_BUFFER_CAPACITY],
                head: 0,
                tail: 0,
                count: 0,
                readers_count: 1,
                writers_count: 1,
                flags,
            };
            return Ok(id);
        }
    }
    Err("Pipe table capacity exhausted")
}

/// Read data bytes from an allocated pipe channel using POSIX stream semantics.
///
/// # Safety
/// Directly accesses `PIPE_TABLE`.
pub unsafe fn read_pipe_channel(pipe_id: u32, buf: &mut [u8]) -> Result<usize, i64> {
    for slot in PIPE_TABLE.iter_mut() {
        if slot.in_use && slot.id == pipe_id {
            if slot.count > 0 {
                let to_read = buf.len().min(slot.count);
                for out in buf.iter_mut().take(to_read) {
                    *out = slot.data[slot.head];
                    slot.head = (slot.head + 1) % PIPE_BUFFER_CAPACITY;
                    slot.count -= 1;
                }
                return Ok(to_read);
            }

            // Buffer is empty: check if writer endpoints remain open
            if slot.writers_count == 0 {
                return Ok(0); // POSIX EOF
            }

            if (slot.flags & O_NONBLOCK) != 0 {
                return Err(-11); // -EAGAIN
            }

            return Ok(0);
        }
    }
    Err(-9) // -EBADF
}

/// Write data bytes into an allocated pipe channel using POSIX stream semantics.
///
/// # Safety
/// Directly accesses `PIPE_TABLE`.
pub unsafe fn write_pipe_channel(pipe_id: u32, buf: &[u8]) -> Result<usize, i64> {
    for slot in PIPE_TABLE.iter_mut() {
        if slot.in_use && slot.id == pipe_id {
            // Broken pipe condition: no active reader endpoints
            if slot.readers_count == 0 {
                return Err(-32); // -EPIPE
            }

            let available = PIPE_BUFFER_CAPACITY - slot.count;
            if available == 0 {
                if (slot.flags & O_NONBLOCK) != 0 {
                    return Err(-11); // -EAGAIN
                }
                return Ok(0);
            }

            let to_write = buf.len().min(available);
            for &byte in buf.iter().take(to_write) {
                slot.data[slot.tail] = byte;
                slot.tail = (slot.tail + 1) % PIPE_BUFFER_CAPACITY;
                slot.count += 1;
            }
            return Ok(to_write);
        }
    }
    Err(-9) // -EBADF
}

/// Decrement reader endpoint refcount and reclaim channel when unreferenced.
///
/// # Safety
/// Directly mutates `PIPE_TABLE`.
pub unsafe fn close_pipe_read(pipe_id: u32) {
    for slot in PIPE_TABLE.iter_mut() {
        if slot.in_use && slot.id == pipe_id {
            if slot.readers_count > 0 {
                slot.readers_count -= 1;
            }
            if slot.readers_count == 0 && slot.writers_count == 0 {
                *slot = PipeChannel::empty();
            }
            return;
        }
    }
}

/// Decrement writer endpoint refcount and reclaim channel when unreferenced.
///
/// # Safety
/// Directly mutates `PIPE_TABLE`.
pub unsafe fn close_pipe_write(pipe_id: u32) {
    for slot in PIPE_TABLE.iter_mut() {
        if slot.in_use && slot.id == pipe_id {
            if slot.writers_count > 0 {
                slot.writers_count -= 1;
            }
            if slot.readers_count == 0 && slot.writers_count == 0 {
                *slot = PipeChannel::empty();
            }
            return;
        }
    }
}

/// Increment reader or writer endpoint refcount on descriptor duplication.
///
/// # Safety
/// Directly mutates `PIPE_TABLE`.
pub unsafe fn dup_pipe_endpoint(pipe_id: u32, is_write: bool) {
    for slot in PIPE_TABLE.iter_mut() {
        if slot.in_use && slot.id == pipe_id {
            if is_write {
                slot.writers_count = slot.writers_count.saturating_add(1);
            } else {
                slot.readers_count = slot.readers_count.saturating_add(1);
            }
            return;
        }
    }
}

/// Retrieve telemetry metrics for active in-kernel pipes.
///
/// # Safety
/// Directly reads `PIPE_TABLE`.
pub unsafe fn get_pipe_stats() -> (usize, usize) {
    let mut active = 0usize;
    let mut buffered = 0usize;
    for slot in PIPE_TABLE.iter() {
        if slot.in_use {
            active += 1;
            buffered += slot.count;
        }
    }
    (active, buffered)
}

/// Retrieve immutable reference to system pipe table.
///
/// # Safety
/// Directly reads `PIPE_TABLE`.
pub unsafe fn get_pipe_table() -> &'static [PipeChannel; MAX_PIPES] {
    &*core::ptr::addr_of!(PIPE_TABLE)
}

/// Reset all pipe channels to empty state (used in testing and teardown).
///
/// # Safety
/// Directly mutates `PIPE_TABLE` and `NEXT_PIPE_ID`.
pub unsafe fn pipe_table_reset() {
    for slot in PIPE_TABLE.iter_mut() {
        *slot = PipeChannel::empty();
    }
    NEXT_PIPE_ID = 1;
}
