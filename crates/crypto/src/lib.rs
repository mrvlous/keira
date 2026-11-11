// SPDX-License-Identifier: GPL-2.0-only
/*
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra <mrvlous@proton.me>
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 */

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
