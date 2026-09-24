// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Fixed-capacity slice buffer adapter implementing `core::fmt::Write`.

use core::fmt::Write;

/// Buffer writer implementing `core::fmt::Write` over a pre-allocated fixed memory slice.
pub struct BufWriter<'a> {
    buf: &'a mut [u8],
    offset: usize,
}

impl<'a> BufWriter<'a> {
    /// Creates a new buffer writer around the specified mutable byte slice.
    pub fn new(buf: &'a mut [u8]) -> Self {
        Self { buf, offset: 0 }
    }

    /// Returns the number of bytes written so far into the underlying slice.
    pub fn len(&self) -> usize {
        self.offset
    }

    /// Checks whether the buffer writer has written zero bytes.
    pub fn is_empty(&self) -> bool {
        self.offset == 0
    }
}

impl<'a> Write for BufWriter<'a> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let bytes = s.as_bytes();
        let avail = self.buf.len().saturating_sub(self.offset);
        let to_write = bytes.len().min(avail);
        self.buf[self.offset..self.offset + to_write].copy_from_slice(&bytes[..to_write]);
        self.offset += to_write;
        Ok(())
    }
}
