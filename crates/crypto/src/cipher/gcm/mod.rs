// SPDX-License-Identifier: GPL-2.0-only

//! Galois/Counter Mode (GCM) authenticated encryption subsystem.

mod core;

pub use self::core::*;

#[cfg(test)]
mod tests;
