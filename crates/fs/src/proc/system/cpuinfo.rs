// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Dynamic `/system/proc/cpuinfo` node reporting CPU architecture and vendor information.

use crate::proc::writer::BufWriter;
use core::fmt::Write;

/// Reads formatted CPU hardware metadata into destination buffer.
pub fn read_cpuinfo(buf: &mut [u8]) -> Result<usize, &'static str> {
    #[cfg(target_arch = "x86_64")]
    let cpuid = core::arch::x86_64::__cpuid(0);
    #[cfg(target_arch = "x86")]
    let cpuid = core::arch::x86::__cpuid(0);
    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    let cpuid = core::arch::x86_64::CpuidResult {
        eax: 0,
        ebx: 0,
        ecx: 0,
        edx: 0,
    };

    let mut vendor = [0u8; 12];
    vendor[0..4].copy_from_slice(&cpuid.ebx.to_le_bytes());
    vendor[4..8].copy_from_slice(&cpuid.edx.to_le_bytes());
    vendor[8..12].copy_from_slice(&cpuid.ecx.to_le_bytes());
    let vendor_str = match core::str::from_utf8(&vendor) {
        Ok(s) => s,
        Err(_) => "UnknownCPU",
    };

    let mut writer = BufWriter::new(buf);
    let _ = write!(
        writer,
        "processor\t: 0\n\
         vendor_id\t: {}\n\
         model name\t: Keira Generic x86 Processor\n\
         cpu MHz\t\t: 2400.000\n\
         flags\t\t: fpu vme de pse tsc msr pae mce cx8 apic sep mtrr pge mca cmov pat pse36 clflush mmx fxsr sse sse2 lm\n",
        vendor_str
    );
    Ok(writer.len())
}
