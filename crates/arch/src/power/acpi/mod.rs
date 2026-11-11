// SPDX-License-Identifier: GPL-2.0-only

//! Advanced Configuration and Power Interface (ACPI) subsystem.

pub mod shutdown;
pub mod tables;

pub use shutdown::*;
pub use tables::*;

#[cfg(test)]
mod tests;
