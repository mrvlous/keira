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

//! Interrupt Descriptor Table (IDT) configuration and dispatch subsystem.

pub mod descriptor;
pub mod handlers;

pub use descriptor::{
    init, load_current_idt, set_gate, IdtEntry, IdtPtr, VECTOR_RESCHEDULE, VECTOR_TLB_SHOOTDOWN,
};
pub use handlers::{irq_get_counter, irq_get_total, isr_handler, IRQ_HIT_COUNTERS};
