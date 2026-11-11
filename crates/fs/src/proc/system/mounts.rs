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

//! Dynamic `/proc/mounts` node reporting registered filesystem mounts.

/// Reads active filesystem mount points into destination buffer.
pub fn read_mounts(buf: &mut [u8]) -> Result<usize, &'static str> {
    let len = crate::vfs::mount::format_proc_mounts(buf);
    Ok(len)
}
