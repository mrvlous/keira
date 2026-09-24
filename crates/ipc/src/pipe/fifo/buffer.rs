// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Ring buffer pipe data storage and system global buffer.

pub const PIPE_BUFFER_SIZE: usize = 1024;

/// In-kernel circular ring buffer for pipe streaming.
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
