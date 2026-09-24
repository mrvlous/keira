// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Pseudo character device `/system/dev/zero` providing infinite null bytes.

/// Reads from `/system/dev/zero`, filling the entire buffer with zeros.
pub fn read(buf: &mut [u8]) -> Result<usize, &'static str> {
    buf.fill(0);
    Ok(buf.len())
}

/// Writes to `/system/dev/zero`, discarding all data and returning bytes written.
pub fn write(buf: &[u8]) -> Result<usize, &'static str> {
    Ok(buf.len())
}
