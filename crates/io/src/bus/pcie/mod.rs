// SPDX-License-Identifier: GPL-2.0-only

//! PCIe Enhanced Configuration Access Mechanism (ECAM) and Message Signaled Interrupts (MSI).

mod ecam;
mod msi;
#[cfg(test)]
mod tests;

pub use self::ecam::*;
pub use self::msi::*;
