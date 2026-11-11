// SPDX-License-Identifier: GPL-2.0-only

//! Hardware devices and peripheral driver shell commands.

pub mod hardware;
pub mod virt;

#[cfg(test)]
mod tests;

pub use hardware::devices;
pub use virt::drivers;
