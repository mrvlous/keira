// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Elliptic curve cryptography (Curve25519 / X25519).

pub mod x25519;

pub use self::x25519::*;

/// Compatibility re-export module for `curve::curve25519`.
pub mod curve25519 {
    pub use super::x25519::*;
}
