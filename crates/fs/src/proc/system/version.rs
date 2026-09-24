// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Dynamic `/system/proc/version` node reporting kernel release and toolchain metadata.

use crate::proc::writer::BufWriter;
use core::fmt::Write;

/// Reads formatted kernel version string into destination buffer.
pub fn read_version(buf: &mut [u8]) -> Result<usize, &'static str> {
    #[cfg(target_arch = "x86_64")]
    const ARCH: &str = "x86_64-elf";
    #[cfg(target_arch = "x86")]
    const ARCH: &str = "i686-elf";
    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    const ARCH: &str = "unknown";

    let mut writer = BufWriter::new(buf);
    let _ = write!(
        writer,
        "Keira Kernel version 0.4.0 ({}) #1 SMP 2026 gcc (Freestanding) rustc\n",
        ARCH
    );
    Ok(writer.len())
}
