// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! AES-128 block cipher and Galois/Counter Mode (GCM) AEAD encryption.

pub mod aes;
pub mod gcm;

pub use aes::Aes128;
pub use gcm::{aes128_gcm_decrypt, aes128_gcm_encrypt};
