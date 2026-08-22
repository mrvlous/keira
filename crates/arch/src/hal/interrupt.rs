// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Architecture-independent Interrupt Controller traits.

/// Generic Programmable Interrupt Controller trait.
pub trait InterruptController {
    /// Initialize interrupt controller routing and priority registers.
    fn init(&mut self);

    /// Signal End of Interrupt (EOI) for the specified IRQ vector.
    fn send_eoi(&mut self, irq: u8);

    /// Mask (disable) hardware interrupt line.
    fn mask_irq(&mut self, irq: u8);

    /// Unmask (enable) hardware interrupt line.
    fn unmask_irq(&mut self, irq: u8);
}
