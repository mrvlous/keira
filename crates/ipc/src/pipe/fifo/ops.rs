// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Pipe read, write, and descriptor creation operations.

use crate::pipe::fifo::buffer::{PIPE_BUFFER_SIZE, SYSTEM_PIPE};

/// Write data bytes into the kernel IPC pipe buffer.
///
/// # Safety
/// Caller must ensure single-threaded kernel execution or cooperative task context.
pub unsafe fn write_pipe(buf: &[u8]) -> usize {
    let pipe_ptr = &raw mut SYSTEM_PIPE;
    let pipe = &mut *pipe_ptr;
    let mut written = 0usize;

    for &b in buf {
        if pipe.count >= PIPE_BUFFER_SIZE {
            break;
        }
        pipe.data[pipe.write_pos] = b;
        pipe.write_pos = (pipe.write_pos + 1) % PIPE_BUFFER_SIZE;
        pipe.count += 1;
        written += 1;
    }

    written
}

/// Read available data bytes from the kernel IPC pipe buffer.
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
        pipe.read_pos = (pipe.read_pos + 1) % PIPE_BUFFER_SIZE;
        pipe.count -= 1;
        read_bytes += 1;
    }

    read_bytes
}

/// Create a new pipe descriptor pair (sys_pipe Vector 23).
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
