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

//! Pseudo character device `/dev/null` discarding writes and returning EOF on reads.

/// Reads from `/dev/null`, instantly returning 0 bytes (EOF).
pub fn read(_buf: &mut [u8]) -> Result<usize, &'static str> {
    Ok(0)
}

/// Writes to `/dev/null`, discarding all data and returning bytes written.
pub fn write(buf: &[u8]) -> Result<usize, &'static str> {
    Ok(buf.len())
}
