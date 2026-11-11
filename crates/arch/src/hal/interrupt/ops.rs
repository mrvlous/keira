// SPDX-License-Identifier: GPL-2.0-only

//! Architecture-independent Programmable Interrupt Controller (PIC) interfaces.
//!
//! Provides abstract routing, masking and End-of-Interrupt (EOI) contracts
//! implemented across Dual 8259 PICs, Local APICs and I/O APICs.

/// Generic Programmable Interrupt Controller trait.
pub trait InterruptController {
    /// Initializes interrupt controller routing, vector offsets and priority registers.
    fn init(&mut self);

    /// Signals End of Interrupt (EOI) for the specified IRQ vector.
    fn send_eoi(&mut self, irq: u8);

    /// Masks (disables) a specific hardware interrupt line.
    fn mask_irq(&mut self, irq: u8);

    /// Unmasks (enables) a specific hardware interrupt line.
    fn unmask_irq(&mut self, irq: u8);
}
