// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Interrupt controllers, Local APIC, Dual 8259 PIC, IDT, and SMP.

pub mod apic;
pub mod idt;
pub mod pic;
pub mod smp;

pub use apic::*;
pub use idt::{isr_handler, set_gate, IdtEntry, IdtPtr};
pub use pic::{clear_mask as pic_clear_mask, send_eoi as pic_send_eoi, set_mask as pic_set_mask};
pub use smp::*;
