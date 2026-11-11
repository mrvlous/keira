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

//! Interrupt controllers, Local APIC, Dual 8259 PIC, IDT and SMP subsystem.
//!
//! Subdivided into dedicated modules for Local APIC, Interrupt Descriptor Table,
//! Dual 8259 PIC and Symmetric Multiprocessing Inter-Processor Interrupts.

pub mod apic;
pub mod idt;
pub mod pic;
pub mod smp;

pub use apic::*;
pub use idt::{
    irq_get_counter, irq_get_total, isr_handler, load_current_idt, set_gate, IdtEntry, IdtPtr,
    IRQ_HIT_COUNTERS,
};
pub use pic::{clear_mask as pic_clear_mask, send_eoi as pic_send_eoi, set_mask as pic_set_mask};
pub use smp::*;
