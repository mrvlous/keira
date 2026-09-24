// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Console output redirection and piping buffers.

/// Flag indicating whether console output is redirected into an in-memory buffer.
pub static mut REDIRECT_TO_FILE: bool = false;

/// In-memory storage buffer for file-redirected console output.
pub static mut REDIRECT_BUFFER: [u8; 4096] = [0; 4096];

/// Number of bytes currently written into the redirection buffer.
pub static mut REDIRECT_LEN: usize = 0;

/// Ring buffer for piping shell command inputs and outputs.
pub static mut PIPE_BUFFER: [u8; 4096] = [0; 4096];

/// Number of active bytes waiting in the pipe buffer.
pub static mut PIPE_LEN: usize = 0;

/// Flag indicating whether pipeline redirection mode is active.
pub static mut PIPE_ACTIVE: bool = false;

/// Current read cursor index within the pipe ring buffer.
pub static mut PIPE_READ_INDEX: usize = 0;
