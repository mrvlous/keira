// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Symmetric Multiprocessing (SMP), Inter-Processor Interrupts (IPI), and cross-core TLB shootdown.

use crate::cpu::invlpg;
use crate::interrupts::apic;

/// Send an Inter-Processor Interrupt (IPI) to a specific target APIC CPU core.
pub fn send_ipi(dest_apic_id: u8, vector: u8) {
    unsafe {
        // ICR High: Destination Field (bits 24..31)
        apic::write_reg(apic::LAPIC_ICR_HIGH_REG, (dest_apic_id as u32) << 24);
        // ICR Low: Delivery Mode Fixed (0), Edge Triggered (0), Vector (bits 0..7)
        apic::write_reg(apic::LAPIC_ICR_LOW_REG, vector as u32);
    }
}

/// Execute cross-core TLB Shootdown to invalidate page address across all CPU cores.
pub fn tlb_shootdown(vaddr: u64) {
    invlpg(vaddr as usize);
}
