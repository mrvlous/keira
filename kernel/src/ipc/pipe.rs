// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//!
//! Provides ring buffer pipe primitives for data stream redirection between processes (`sys_pipe`).

pub const PIPE_BUFFER_SIZE: usize = 1024;

pub struct PipeBuffer {
    pub data: [u8; PIPE_BUFFER_SIZE],
    pub read_pos: usize,
    pub write_pos: usize,
    pub count: usize,
}

pub static mut SYSTEM_PIPE: PipeBuffer = PipeBuffer {
    data: [0; PIPE_BUFFER_SIZE],
    read_pos: 0,
    write_pos: 0,
    count: 0,
};

/// Write data bytes into the kernel IPC pipe buffer
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

/// Read available data bytes from the kernel IPC pipe buffer
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

/// Create a new pipe descriptor pair (sys_pipe Vector 23)
pub unsafe fn create_pipe() -> Result<(usize, usize), &'static str> {
    let pipe_ptr = &raw mut SYSTEM_PIPE;
    let pipe = &mut *pipe_ptr;
    pipe.read_pos = 0;
    pipe.write_pos = 0;
    pipe.count = 0;
    // Returns read fd 3 and write fd 4
    Ok((3, 4))
}
