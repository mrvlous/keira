// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Pseudo character devices `/system/dev/random` and `/system/dev/urandom` providing entropy.

/// Fills destination buffer with pseudo-random entropy derived from CPU Time-Stamp Counter (TSC).
pub fn read(buf: &mut [u8]) -> Result<usize, &'static str> {
    let lo: u32;
    let hi: u32;
    #[cfg(target_arch = "x86_64")]
    unsafe {
        core::arch::asm!("rdtsc", out("eax") lo, out("edx") hi, options(nomem, nostack));
    }
    #[cfg(target_arch = "x86")]
    unsafe {
        core::arch::asm!("rdtsc", out("eax") lo, out("edx") hi, options(nomem, nostack));
    }
    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    {
        lo = 0x1234_5678;
        hi = 0x9ABC_DEF0;
    }

    let tsc = ((hi as u64) << 32) | (lo as u64);
    for (idx, slot) in buf.iter_mut().enumerate() {
        let shift = (idx % 8) * 8;
        let byte = ((tsc >> shift) ^ (idx as u64 * 0x9E37_79B9)) as u8;
        *slot = byte;
    }
    Ok(buf.len())
}

/// Discards entropy seed writes to random/urandom.
pub fn write(buf: &[u8]) -> Result<usize, &'static str> {
    Ok(buf.len())
}
