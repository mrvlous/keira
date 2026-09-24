// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! File descriptor handles, socket associations, and pipe descriptors.

/// Maximum number of file descriptors per process.
pub const MAX_FDS: usize = 32;

/// Process file descriptor handle.
#[derive(Clone, Copy, Debug)]
pub struct FileDescriptor {
    pub is_open: bool,
    pub path: [u8; 128],
    pub path_len: usize,
    pub offset: u64,
    pub write_mode: bool,
    pub is_socket: bool,
    pub socket_id: u32,
    pub nonblocking: bool,
    pub is_pipe: bool,
    pub pipe_write: bool,
}

impl FileDescriptor {
    /// Allocate an empty, closed file descriptor descriptor.
    pub const fn new() -> Self {
        Self {
            is_open: false,
            path: [0u8; 128],
            path_len: 0,
            offset: 0,
            write_mode: false,
            is_socket: false,
            socket_id: 0,
            nonblocking: false,
            is_pipe: false,
            pipe_write: false,
        }
    }

    /// Construct an open socket file descriptor.
    pub fn new_socket(socket_id: u32, nonblocking: bool) -> Self {
        Self {
            is_open: true,
            path: [0u8; 128],
            path_len: 0,
            offset: 0,
            write_mode: true,
            is_socket: true,
            socket_id,
            nonblocking,
            is_pipe: false,
            pipe_write: false,
        }
    }

    /// Construct an open pipe file descriptor.
    pub fn new_pipe(write_mode: bool) -> Self {
        Self {
            is_open: true,
            path: [0u8; 128],
            path_len: 0,
            offset: 0,
            write_mode,
            is_socket: false,
            socket_id: 0,
            nonblocking: false,
            is_pipe: true,
            pipe_write: write_mode,
        }
    }
}

impl Default for FileDescriptor {
    fn default() -> Self {
        Self::new()
    }
}
