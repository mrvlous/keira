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

//! Formatted string buffer writer for ProcFS synthetic files.

pub mod buf {
    use core::fmt::Write;

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
}

pub use self::buf::*;
