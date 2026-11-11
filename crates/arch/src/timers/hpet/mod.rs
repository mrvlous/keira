// SPDX-License-Identifier: GPL-2.0-only

//! High-Precision Event Timer (HPET) subsystem.

pub mod controller;

pub use controller::*;

#[cfg(test)]
mod tests;
