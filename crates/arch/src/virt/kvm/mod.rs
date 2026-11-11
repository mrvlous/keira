// SPDX-License-Identifier: GPL-2.0-only

//! Hardware-assisted virtualization and Kernel-based Virtual Machine (KVM) subsystem.

pub mod engine;

pub use engine::*;

#[cfg(test)]
mod tests;
