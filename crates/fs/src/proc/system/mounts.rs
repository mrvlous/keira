// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Dynamic `/proc/mounts` node reporting registered filesystem mounts.

use crate::proc::writer::BufWriter;
use core::fmt::Write;

/// Reads active filesystem mount points into destination buffer.
pub fn read_mounts(buf: &mut [u8]) -> Result<usize, &'static str> {
    let mut writer = BufWriter::new(buf);

    if let Some(dev) = keira_io::storage::block::get_mounted_device() {
        let _ = writeln!(writer, "/dev/{} / fat16 rw,relatime 0 0", dev.get_name());
    } else {
        let _ = writeln!(writer, "rootfs / rootfs rw,relatime 0 0");
    }

    let _ = writeln!(writer, "devfs /dev devfs rw,nosuid,relatime 0 0");
    let _ = writeln!(
        writer,
        "procfs /proc procfs rw,nosuid,nodev,noexec,relatime 0 0"
    );

    if unsafe { crate::tar::INITRD_START != 0 && crate::tar::INITRD_END > crate::tar::INITRD_START }
    {
        let _ = writeln!(writer, "initrd /initrd tar ro,relatime 0 0");
    }

    Ok(writer.len())
}
