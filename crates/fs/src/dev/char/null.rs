// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Pseudo character device `/system/dev/null` discarding writes and returning EOF on reads.

/// Reads from `/system/dev/null`, instantly returning 0 bytes (EOF).
pub fn read(_buf: &mut [u8]) -> Result<usize, &'static str> {
    Ok(0)
}

/// Writes to `/system/dev/null`, discarding all data and returning bytes written.
pub fn write(buf: &[u8]) -> Result<usize, &'static str> {
    Ok(buf.len())
}
