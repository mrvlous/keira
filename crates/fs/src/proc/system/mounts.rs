// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Dynamic `/proc/mounts` node reporting registered filesystem mounts.

/// Reads active filesystem mount points into destination buffer.
pub fn read_mounts(buf: &mut [u8]) -> Result<usize, &'static str> {
    let len = crate::vfs::mount::format_proc_mounts(buf);
    Ok(len)
}
