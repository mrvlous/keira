// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Galois/Counter Mode (GCM) Authenticated Encryption with Associated Data (AEAD).

use super::aes::Aes128;

#[inline(always)]
fn gcm_inc32(counter: &mut [u8; 16]) {
    let mut val = u32::from_be_bytes([counter[12], counter[13], counter[14], counter[15]]);
    val = val.wrapping_add(1);
    counter[12..16].copy_from_slice(&val.to_be_bytes());
}

fn ghash_multiply(x: &[u8; 16], h: &[u8; 16]) -> [u8; 16] {
    let mut z = [0u8; 16];
    let mut v = *h;

    for i in 0..128 {
        let byte_idx = i / 8;
        let bit_idx = 7 - (i % 8);

        if (x[byte_idx] >> bit_idx) & 1 == 1 {
            for j in 0..16 {
                z[j] ^= v[j];
            }
        }

        let msb = v[15] & 1;
        for j in (1..16).rev() {
            v[j] = (v[j] >> 1) | (v[j - 1] << 7);
        }
        v[0] >>= 1;

        if msb == 1 {
            v[0] ^= 0xe1;
        }
    }

    z
}

fn ghash(h: &[u8; 16], aad: &[u8], ciphertext: &[u8]) -> [u8; 16] {
    let mut tag = [0u8; 16];

    let mut offset = 0;
    while offset < aad.len() {
        let mut block = [0u8; 16];
        let end = core::cmp::min(offset + 16, aad.len());
        block[..end - offset].copy_from_slice(&aad[offset..end]);
        for i in 0..16 {
            tag[i] ^= block[i];
        }
        tag = ghash_multiply(&tag, h);
        offset += 16;
    }

    offset = 0;
    while offset < ciphertext.len() {
        let mut block = [0u8; 16];
        let end = core::cmp::min(offset + 16, ciphertext.len());
        block[..end - offset].copy_from_slice(&ciphertext[offset..end]);
        for i in 0..16 {
            tag[i] ^= block[i];
        }
        tag = ghash_multiply(&tag, h);
        offset += 16;
    }

    let mut len_block = [0u8; 16];
    len_block[..8].copy_from_slice(&((aad.len() as u64) * 8).to_be_bytes());
    len_block[8..].copy_from_slice(&((ciphertext.len() as u64) * 8).to_be_bytes());
    for i in 0..16 {
        tag[i] ^= len_block[i];
    }
    tag = ghash_multiply(&tag, h);

    tag
}

/// Encrypt plaintext using AES-128-GCM and generate 16-byte authentication tag.
pub fn aes128_gcm_encrypt(
    key: &[u8; 16],
    iv: &[u8; 12],
    plaintext: &[u8],
    aad: &[u8],
    out: &mut [u8],
) -> [u8; 16] {
    let cipher = Aes128::new(key);

    let mut h_block = [0u8; 16];
    cipher.encrypt_block(&mut h_block);

    let mut j0 = [0u8; 16];
    j0[..12].copy_from_slice(iv);
    j0[12..16].copy_from_slice(&1u32.to_be_bytes());

    let mut counter = j0;
    let mut offset = 0;
    while offset < plaintext.len() {
        gcm_inc32(&mut counter);
        let mut keystream = counter;
        cipher.encrypt_block(&mut keystream);

        let end = core::cmp::min(offset + 16, plaintext.len());
        for i in offset..end {
            out[i] = plaintext[i] ^ keystream[i - offset];
        }
        offset += 16;
    }

    let ct_len = plaintext.len();
    let mut tag = ghash(&h_block, aad, &out[..ct_len]);

    let mut e_j0 = j0;
    cipher.encrypt_block(&mut e_j0);
    for i in 0..16 {
        tag[i] ^= e_j0[i];
    }

    tag
}

/// Decrypt ciphertext using AES-128-GCM and verify the 16-byte authentication tag.
pub fn aes128_gcm_decrypt(
    key: &[u8; 16],
    iv: &[u8; 12],
    ciphertext: &[u8],
    aad: &[u8],
    tag: &[u8; 16],
    out: &mut [u8],
) -> bool {
    let cipher = Aes128::new(key);

    let mut h_block = [0u8; 16];
    cipher.encrypt_block(&mut h_block);

    let mut j0 = [0u8; 16];
    j0[..12].copy_from_slice(iv);
    j0[12..16].copy_from_slice(&1u32.to_be_bytes());

    let mut counter = j0;
    let mut offset = 0;
    while offset < ciphertext.len() {
        gcm_inc32(&mut counter);
        let mut keystream = counter;
        cipher.encrypt_block(&mut keystream);

        let end = core::cmp::min(offset + 16, ciphertext.len());
        for i in offset..end {
            out[i] = ciphertext[i] ^ keystream[i - offset];
        }
        offset += 16;
    }

    let mut calc_tag = ghash(&h_block, aad, ciphertext);

    let mut e_j0 = j0;
    cipher.encrypt_block(&mut e_j0);
    for i in 0..16 {
        calc_tag[i] ^= e_j0[i];
    }

    let mut diff = 0u8;
    for i in 0..16 {
        diff |= calc_tag[i] ^ tag[i];
    }

    diff == 0
}
