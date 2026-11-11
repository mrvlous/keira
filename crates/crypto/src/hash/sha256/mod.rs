// SPDX-License-Identifier: GPL-2.0-only

//! Secure Hash Algorithm 256 (SHA-256) subsystem.

mod engine;

pub use self::engine::*;

#[cfg(test)]
mod tests;
