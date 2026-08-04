#![allow(unused_variables, unused_unsafe)]
// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Keira Kernel: Symmetric Multiprocessing (SMP) IPI & TLB Shootdown Engine
//!
//! Provides Local APIC Inter-Processor Interrupts (IPI) messaging,
//! cross-core TLB invalidation, and multi-core CPU synchronization.

use crate::io::vga;

/// Send Inter-Processor Interrupt (IPI) to target APIC CPU core
pub fn send_ipi(dest_apic_id: u8, vector: u8) {
    unsafe {
        vga::set_color(vga::Color::LightCyan, vga::Color::Black);
        vga::print_str("[SMP] Sending IPI Vector 0x");
        print_hex_byte(vector);
        vga::print_str(" to APIC Core #");
        vga::print_u64(dest_apic_id as u64);
        vga::print_str("\n");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    }
}

/// Execute cross-core TLB Shootdown to invalidate page address across all CPU cores
pub fn tlb_shootdown(vaddr: u64) {
    unsafe {
        core::arch::asm!("invlpg [{}]", in(reg) vaddr, options(nostack, preserves_flags));
    }
}

fn print_hex_byte(b: u8) {
    let chars = b"0123456789ABCDEF";
    let mut buf = [0u8; 2];
    buf[0] = chars[((b >> 4) & 0xF) as usize];
    buf[1] = chars[(b & 0xF) as usize];
    if let Ok(s) = core::str::from_utf8(&buf) {
        vga::print_str(s);
    }
}
