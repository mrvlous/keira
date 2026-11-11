// SPDX-License-Identifier: GPL-2.0-only

//! Curve25519 and X25519 Diffie-Hellman key exchange subsystem.

pub mod field;
pub mod scalar;

pub use self::field::*;
pub use self::scalar::*;

#[cfg(test)]
mod tests;
