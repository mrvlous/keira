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

//! HMAC-SHA-256 (RFC 2104) keyed-hash message authentication code generator.

use crate::hash::sha256::{sha256, Sha256};

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hmac_sha256_rfc4231_case1() {
        let key = [0x0bu8; 20];
        let data = b"Hi There";
        let expected: [u8; 32] = [
            0xb0, 0x34, 0x4c, 0x61, 0xd8, 0xdb, 0x38, 0x53, 0x5c, 0xa8, 0xaf, 0xce, 0xaf, 0x0b,
            0xf1, 0x2b, 0x88, 0x1d, 0xc2, 0x00, 0xc9, 0x83, 0x3d, 0xa7, 0x26, 0xe9, 0x37, 0x6c,
            0x2e, 0x32, 0xcf, 0xf7,
        ];

        let result = hmac_sha256(&key, data);
        assert_eq!(result, expected);
    }
}
