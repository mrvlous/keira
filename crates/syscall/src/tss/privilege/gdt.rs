// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! GDT descriptor programming and TSS register loading routines.

use crate::tss::segment::stack::TSS;
use crate::tss::segment::types::TaskStateSegment;

#[cfg(not(test))]
extern "C" {
    #[cfg(target_arch = "x86_64")]
    static mut tss_descriptor: [u8; 16];
    #[cfg(target_arch = "x86")]
    static mut tss_descriptor: [u8; 8];

    fn reload_gdt();
    fn load_tss();
}

#[cfg(test)]
static mut TEST_TSS_DESCRIPTOR: [u8; 16] = [0; 16];

#[cfg(test)]
unsafe fn reload_gdt() {}

#[cfg(test)]
unsafe fn load_tss() {}

/// Populates the GDT TSS descriptor entry and flushes processor segment registers.
pub unsafe fn configure_gdt_tss() {
    let tss_addr = &raw const TSS as usize;
    let tss_size = core::mem::size_of::<TaskStateSegment>() - 1;

    #[cfg(not(test))]
    let desc = &raw mut tss_descriptor;
    #[cfg(test)]
    let desc = &raw mut TEST_TSS_DESCRIPTOR;

    #[cfg(target_arch = "x86_64")]
    {
        *(desc.cast::<u16>()) = tss_size as u16;
        *((desc as u64 + 2) as *mut u16) = (tss_addr & 0xFFFF) as u16;
        *((desc as u64 + 4) as *mut u8) = ((tss_addr >> 16) & 0xFF) as u8;
        *((desc as u64 + 5) as *mut u8) = 0x89;
        *((desc as u64 + 6) as *mut u8) = 0x00;
        *((desc as u64 + 7) as *mut u8) = ((tss_addr >> 24) & 0xFF) as u8;
        *((desc as u64 + 8) as *mut u32) = ((tss_addr >> 32) & 0xFFFFFFFF) as u32;
        *((desc as u64 + 12) as *mut u32) = 0;

        reload_gdt();
        load_tss();
    }

    #[cfg(target_arch = "x86")]
    {
        *(desc.cast::<u16>()) = tss_size as u16;
        *((desc as usize + 2) as *mut u16) = (tss_addr & 0xFFFF) as u16;
        *((desc as usize + 4) as *mut u8) = ((tss_addr >> 16) & 0xFF) as u8;
        *((desc as usize + 5) as *mut u8) = 0x89;
        *((desc as usize + 6) as *mut u8) = 0x00;
        *((desc as usize + 7) as *mut u8) = ((tss_addr >> 24) & 0xFF) as u8;

        reload_gdt();
        load_tss();
    }
}
