// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Architecture-independent Programmable Interrupt Controller (PIC) interfaces.
//!
//! Provides abstract routing, masking, and End-of-Interrupt (EOI) contracts
//! implemented across Dual 8259 PICs, Local APICs, and I/O APICs.

/// Generic Programmable Interrupt Controller trait.
pub trait InterruptController {
    /// Initializes interrupt controller routing, vector offsets, and priority registers.
    fn init(&mut self);

    /// Signals End of Interrupt (EOI) for the specified IRQ vector.
    fn send_eoi(&mut self, irq: u8);

    /// Masks (disables) a specific hardware interrupt line.
    fn mask_irq(&mut self, irq: u8);

    /// Unmasks (enables) a specific hardware interrupt line.
    fn unmask_irq(&mut self, irq: u8);
}
