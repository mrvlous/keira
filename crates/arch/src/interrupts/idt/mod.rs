// SPDX-License-Identifier: GPL-2.0-only

//! Interrupt Descriptor Table (IDT) configuration and dispatch subsystem.

pub mod descriptor;
pub mod handlers;

pub use descriptor::{
    init, load_current_idt, set_gate, IdtEntry, IdtPtr, VECTOR_RESCHEDULE, VECTOR_TLB_SHOOTDOWN,
};
pub use handlers::{irq_get_counter, irq_get_total, isr_handler, IRQ_HIT_COUNTERS};
