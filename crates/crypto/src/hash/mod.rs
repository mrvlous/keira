// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Cryptographic hashing algorithms and message authentication.

pub mod hmac;
pub mod sha256;

pub use hmac::hmac_sha256;
pub use sha256::{sha256, Sha256};
