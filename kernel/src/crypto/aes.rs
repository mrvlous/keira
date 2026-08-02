// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Keira Kernel: AES-128 & AES-128-GCM Authenticated Encryption Subsystem
//!
//! Provides bare-metal implementation of AES-128 Block Cipher and
//! Galois/Counter Mode (GCM) Authenticated Encryption with Associated Data (AEAD).

const AES_SBOX: [u8; 256] = [
    0x63, 0x7c, 0x77, 0x7b, 0xf2, 0x6b, 0x6f, 0xc5, 0x30, 0x01, 0x67, 0x2b, 0xfe, 0xd7, 0xab, 0x76,
    0xca, 0x82, 0xc9, 0x7d, 0xfa, 0x59, 0x47, 0xf0, 0xad, 0xd4, 0xa2, 0xaf, 0x9c, 0xa4, 0x72, 0xc0,
    0xb7, 0xfd, 0x93, 0x26, 0x36, 0x3f, 0xf7, 0xcc, 0x34, 0xa5, 0xe5, 0xf1, 0x71, 0xd8, 0x31, 0x15,
    0x04, 0xc7, 0x23, 0xc3, 0x18, 0x96, 0x05, 0x9a, 0x07, 0x12, 0x80, 0xe2, 0xeb, 0x27, 0xb2, 0x75,
    0x09, 0x83, 0x2c, 0x1a, 0x1b, 0x6e, 0x5a, 0xa0, 0x52, 0x3b, 0xd6, 0xb3, 0x29, 0xe3, 0x2f, 0x84,
    0x53, 0xd1, 0x00, 0xed, 0x20, 0xfc, 0xb1, 0x5b, 0x6a, 0xcb, 0xbe, 0x39, 0x4a, 0x4c, 0x58, 0xcf,
    0xd0, 0xef, 0xaa, 0xfb, 0x43, 0x4d, 0x33, 0x85, 0x45, 0xf9, 0x02, 0x7f, 0x50, 0x3c, 0x9f, 0xa8,
    0x51, 0xa3, 0x40, 0x8f, 0x92, 0x9d, 0x38, 0xf5, 0xbc, 0xb6, 0xda, 0x21, 0x10, 0xff, 0xf3, 0xd2,
    0xcd, 0x0c, 0x13, 0xec, 0x5f, 0x97, 0x44, 0x17, 0xc4, 0xa7, 0x7e, 0x3d, 0x64, 0x5d, 0x19, 0x73,
    0x60, 0x81, 0x4f, 0xdc, 0x22, 0x2a, 0x90, 0x88, 0x46, 0xee, 0xb8, 0x14, 0xde, 0x5e, 0x0b, 0xdb,
    0xe0, 0x32, 0x3a, 0x0a, 0x49, 0x06, 0x24, 0x5c, 0xc2, 0xd3, 0xac, 0x62, 0x91, 0x95, 0xe4, 0x79,
    0xe7, 0xc8, 0x37, 0x6d, 0x8d, 0xd5, 0x4e, 0xa9, 0x6c, 0x56, 0xf4, 0xea, 0x65, 0x7a, 0xae, 0x08,
    0xba, 0x78, 0x25, 0x2e, 0x1c, 0xa6, 0xb4, 0xc6, 0xe8, 0xdd, 0x74, 0x1f, 0x4b, 0xbd, 0x8b, 0x8a,
    0x70, 0x3e, 0xb5, 0x66, 0x48, 0x03, 0xf6, 0x0e, 0x61, 0x35, 0x57, 0xb9, 0x86, 0xc1, 0x1d, 0x9e,
    0xe1, 0xf8, 0x98, 0x11, 0x69, 0xd9, 0x8e, 0x94, 0x9b, 0x1e, 0x87, 0xe9, 0xce, 0x55, 0x28, 0xdf,
    0x8c, 0xa1, 0x89, 0x0d, 0xbf, 0xe6, 0x42, 0x68, 0x41, 0x99, 0x2d, 0x0f, 0xb0, 0x54, 0xbb, 0x16,
];

const AES_RCON: [u8; 10] = [0x01, 0x02, 0x04, 0x08, 0x10, 0x20, 0x40, 0x80, 0x1b, 0x36];

pub struct Aes128 {
    round_keys: [u8; 176],
}

impl Aes128 {
    pub fn new(key: &[u8; 16]) -> Self {
        let mut rk = [0u8; 176];
        rk[..16].copy_from_slice(key);

        for i in 0..10 {
            let prev = i * 16;
            let next = (i + 1) * 16;

            let temp = [
                AES_SBOX[rk[prev + 13] as usize] ^ AES_RCON[i],
                AES_SBOX[rk[prev + 14] as usize],
                AES_SBOX[rk[prev + 15] as usize],
                AES_SBOX[rk[prev + 12] as usize],
            ];

            for j in 0..4 {
                rk[next + j] = rk[prev + j] ^ temp[j];
            }
            for j in 4..16 {
                rk[next + j] = rk[prev + j] ^ rk[next + j - 4];
            }
        }

        Self { round_keys: rk }
    }

    pub fn encrypt_block(&self, block: &mut [u8; 16]) {
        for i in 0..16 {
            block[i] ^= self.round_keys[i];
        }

        for round in 1..10 {
            for i in 0..16 {
                block[i] = AES_SBOX[block[i] as usize];
            }

            let tmp = block[1];
            block[1] = block[5];
            block[5] = block[9];
            block[9] = block[13];
            block[13] = tmp;

            let tmp = block[2];
            block[2] = block[10];
            block[10] = tmp;
            let tmp = block[6];
            block[6] = block[14];
            block[14] = tmp;

            let tmp = block[3];
            block[3] = block[15];
            block[15] = block[11];
            block[11] = block[7];
            block[7] = tmp;

            for col in 0..4 {
                let c = col * 4;
                let a0 = block[c];
                let a1 = block[c + 1];
                let a2 = block[c + 2];
                let a3 = block[c + 3];
                block[c] = gf_mul2(a0) ^ gf_mul3(a1) ^ a2 ^ a3;
                block[c + 1] = a0 ^ gf_mul2(a1) ^ gf_mul3(a2) ^ a3;
                block[c + 2] = a0 ^ a1 ^ gf_mul2(a2) ^ gf_mul3(a3);
                block[c + 3] = gf_mul3(a0) ^ a1 ^ a2 ^ gf_mul2(a3);
            }

            let rk_offset = round * 16;
            for i in 0..16 {
                block[i] ^= self.round_keys[rk_offset + i];
            }
        }

        for i in 0..16 {
            block[i] = AES_SBOX[block[i] as usize];
        }

        let tmp = block[1];
        block[1] = block[5];
        block[5] = block[9];
        block[9] = block[13];
        block[13] = tmp;

        let tmp = block[2];
        block[2] = block[10];
        block[10] = tmp;
        let tmp = block[6];
        block[6] = block[14];
        block[14] = tmp;

        let tmp = block[3];
        block[3] = block[15];
        block[15] = block[11];
        block[11] = block[7];
        block[7] = tmp;

        for i in 0..16 {
            block[i] ^= self.round_keys[160 + i];
        }
    }
}

fn gf_mul2(a: u8) -> u8 {
    let shifted = (a as u16) << 1;
    let result = if a & 0x80 != 0 {
        shifted ^ 0x11b
    } else {
        shifted
    };
    result as u8
}

fn gf_mul3(a: u8) -> u8 {
    gf_mul2(a) ^ a
}

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
