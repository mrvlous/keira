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
#[repr(align(16))]
struct TestTssDescriptor([u8; 16]);

#[cfg(test)]
static mut TEST_TSS_DESCRIPTOR: TestTssDescriptor = TestTssDescriptor([0; 16]);

#[cfg(test)]
unsafe fn reload_gdt() {}

#[cfg(test)]
unsafe fn load_tss() {}

/// Populates the GDT TSS descriptor entry and flushes processor segment registers.
///
/// # Safety
/// Modifies low-level Global Descriptor Table descriptor entries and executes segment reload instructions.
pub unsafe fn configure_gdt_tss() {
    let tss_addr = &raw const TSS as usize;
    let tss_size = core::mem::size_of::<TaskStateSegment>() - 1;

    #[cfg(not(test))]
    let desc = (&raw mut tss_descriptor) as *mut u8;
    #[cfg(test)]
    let desc = core::ptr::addr_of_mut!(TEST_TSS_DESCRIPTOR.0).cast::<u8>();

    #[cfg(target_arch = "x86_64")]
    {
        core::ptr::write_unaligned(desc.cast::<u16>(), tss_size as u16);
        core::ptr::write_unaligned((desc.add(2)).cast::<u16>(), (tss_addr & 0xFFFF) as u16);
        core::ptr::write_unaligned(desc.add(4), ((tss_addr >> 16) & 0xFF) as u8);
        core::ptr::write_unaligned(desc.add(5), 0x89);
        core::ptr::write_unaligned(desc.add(6), 0x00);
        core::ptr::write_unaligned(desc.add(7), ((tss_addr >> 24) & 0xFF) as u8);
        core::ptr::write_unaligned(
            (desc.add(8)).cast::<u32>(),
            ((tss_addr >> 32) & 0xFFFFFFFF) as u32,
        );
        core::ptr::write_unaligned((desc.add(12)).cast::<u32>(), 0);

        reload_gdt();
        load_tss();
    }

    #[cfg(target_arch = "x86")]
    {
        core::ptr::write_unaligned(desc.cast::<u16>(), tss_size as u16);
        core::ptr::write_unaligned((desc.add(2)).cast::<u16>(), (tss_addr & 0xFFFF) as u16);
        core::ptr::write_unaligned(desc.add(4), ((tss_addr >> 16) & 0xFF) as u8);
        core::ptr::write_unaligned(desc.add(5), 0x89);
        core::ptr::write_unaligned(desc.add(6), 0x00);
        core::ptr::write_unaligned(desc.add(7), ((tss_addr >> 24) & 0xFF) as u8);

        reload_gdt();
        load_tss();
    }
}
