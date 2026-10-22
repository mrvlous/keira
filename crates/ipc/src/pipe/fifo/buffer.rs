// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Dynamic anonymous pipe buffer structures and multi-channel table.

#![allow(static_mut_refs)]

/// Maximum number of concurrent anonymous pipe channels in kernel space.
pub const MAX_PIPES: usize = 32;

/// Standard ring buffer capacity per anonymous pipe (4 KiB page size).
pub const PIPE_BUFFER_CAPACITY: usize = 4096;

/// Legacy alias for single-channel buffer size.
pub const PIPE_BUFFER_SIZE: usize = PIPE_BUFFER_CAPACITY;

/// Non-blocking I/O flag for pipe creation.
pub const O_NONBLOCK: u32 = 0x800;

/// Close-on-exec flag for pipe descriptor creation.
pub const O_CLOEXEC: u32 = 0x80000;

/// In-kernel circular ring buffer for pipe streaming.
pub struct PipeBuffer {
    pub data: [u8; 1024],
    pub read_pos: usize,
    pub write_pos: usize,
    pub count: usize,
}

/// Legacy global system pipe for backward compatibility.
pub static mut SYSTEM_PIPE: PipeBuffer = PipeBuffer {
    data: [0; 1024],
    read_pos: 0,
    write_pos: 0,
    count: 0,
};

/// Descriptor representing an allocated in-kernel anonymous pipe channel.
#[derive(Copy, Clone, Debug)]
pub struct PipeChannel {
    pub id: u32,
    pub in_use: bool,
    pub data: [u8; PIPE_BUFFER_CAPACITY],
    pub head: usize,
    pub tail: usize,
    pub count: usize,
    pub readers_count: u32,
    pub writers_count: u32,
    pub flags: u32,
}

impl PipeChannel {
    /// Const constructor for an unallocated pipe channel slot.
    pub const fn empty() -> Self {
        Self {
            id: 0,
            in_use: false,
            data: [0u8; PIPE_BUFFER_CAPACITY],
            head: 0,
            tail: 0,
            count: 0,
            readers_count: 0,
            writers_count: 0,
            flags: 0,
        }
    }
}

/// System-wide registry of active anonymous pipe channels.
pub static mut PIPE_TABLE: [PipeChannel; MAX_PIPES] = [PipeChannel::empty(); MAX_PIPES];

/// Monotonically increasing identifier counter for allocated pipe channels.
pub static mut NEXT_PIPE_ID: u32 = 1;
