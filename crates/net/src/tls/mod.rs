// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Transport Layer Security (TLS 1.3) cryptographic record and session state machine.

pub mod handshake;
pub mod record;

pub use handshake::{build_hkdf_label, tls_connect};
pub use record::*;

pub mod native {
    pub use super::handshake::{build_hkdf_label, tls_connect};
    pub use super::record::*;
}

#[cfg(test)]
mod tests;
