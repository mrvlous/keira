// SPDX-License-Identifier: GPL-2.0-only

//! Pure freestanding cryptographic primitives (SHA-256, HMAC, AES-128-GCM, Curve25519, TPM 2.0).

#![no_std]
#![allow(clippy::needless_range_loop)]

pub mod cipher;
pub mod curve;
pub mod hash;
pub mod tpm;

pub use self::cipher::*;
pub use self::curve::*;
pub use self::hash::*;
pub use self::tpm::*;

#[cfg(test)]
mod tests;
