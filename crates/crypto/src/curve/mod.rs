// SPDX-License-Identifier: GPL-2.0-only

//! Elliptic curve cryptography (Curve25519 / X25519).

pub mod x25519;

pub use self::x25519::*;

/// Compatibility re-export module for `curve::curve25519`.
pub mod curve25519 {
    pub use super::x25519::*;
}
