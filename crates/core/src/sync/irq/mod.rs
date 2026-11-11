// SPDX-License-Identifier: GPL-2.0-only

//! Interrupt state management and critical section masking subsystem.

pub mod state;

pub use state::{interrupts_enabled, irq_restore, irq_save, IrqState};
