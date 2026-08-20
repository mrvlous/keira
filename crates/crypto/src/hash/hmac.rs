// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! HMAC-SHA-256 (RFC 2104) keyed-hash message authentication code generator.

use super::sha256::{sha256, Sha256};

/// Compute HMAC-SHA-256 keyed-hash message authentication code.
pub fn hmac_sha256(key: &[u8], message: &[u8]) -> [u8; 32] {
    let mut padded_key = [0u8; 64];

    if key.len() > 64 {
        let hashed = sha256(key);
        padded_key[..32].copy_from_slice(&hashed);
    } else {
        padded_key[..key.len()].copy_from_slice(key);
    }

    let mut i_key_pad = [0x36u8; 64];
    for i in 0..64 {
        i_key_pad[i] ^= padded_key[i];
    }

    let mut o_key_pad = [0x5cu8; 64];
    for i in 0..64 {
        o_key_pad[i] ^= padded_key[i];
    }

    let mut inner_hasher = Sha256::new();
    inner_hasher.update(&i_key_pad);
    inner_hasher.update(message);
    let inner_hash = inner_hasher.finalize();

    let mut outer_hasher = Sha256::new();
    outer_hasher.update(&o_key_pad);
    outer_hasher.update(&inner_hash);
    outer_hasher.finalize()
}
