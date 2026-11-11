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

//! Dynamic `/proc/net/unix` node reporting active kernel UNIX domain sockets.

use crate::proc::system::hooks::NET_UNIX_HOOK;
use crate::proc::writer::BufWriter;
use core::fmt::Write;

/// Reads formatted active UNIX domain socket table into destination buffer.
pub fn read_net_unix(buf: &mut [u8]) -> Result<usize, &'static str> {
    if let Some(hook) = unsafe { NET_UNIX_HOOK } {
        return Ok(hook(buf));
    }

    let mut writer = BufWriter::new(buf);

    let _ = writeln!(
        writer,
        "Num       RefCount Protocol Flags    Type St Inode Path"
    );

    Ok(writer.len())
}
