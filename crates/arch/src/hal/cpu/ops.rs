// SPDX-License-Identifier: GPL-2.0-only

//! Architecture-independent CPU execution control interfaces.
//!
//! Exposes generic processor management traits providing uniform primitives
//! for halting, interrupt masking, core enumeration and pipeline spin hints.

/// Generic CPU hardware operations trait.
pub trait Cpu {
    /// Halts processor execution until the next hardware interrupt arrives.
    fn halt(&self);

    /// Atomically enables hardware interrupts on the executing core.
    fn enable_interrupts(&self);

    /// Atomically disables hardware interrupts on the executing core.
    fn disable_interrupts(&self);

    /// Queries whether hardware interrupts are currently enabled on this core.
    fn interrupts_enabled(&self) -> bool;

    /// Retrieves the hardware core ID of the executing CPU.
    fn cpu_id(&self) -> u32;

    /// Executes an architecture-specific pause or spin-loop hint instruction.
    fn pause(&self);
}
