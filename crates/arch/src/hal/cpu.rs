// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Architecture-independent CPU control and execution traits.

/// Generic CPU hardware operations trait.
pub trait Cpu {
    /// Halt the CPU until the next interrupt arrives.
    fn halt(&self);

    /// Atomically enable hardware interrupts.
    fn enable_interrupts(&self);

    /// Atomically disable hardware interrupts.
    fn disable_interrupts(&self);

    /// Check if hardware interrupts are currently enabled.
    fn interrupts_enabled(&self) -> bool;

    /// Retrieve the hardware core ID of the executing CPU.
    fn cpu_id(&self) -> u32;

    /// Execute an architecture-specific pause/yield instruction.
    fn pause(&self);
}
