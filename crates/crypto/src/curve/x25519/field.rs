// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Galois field GF(2^255 - 19) arithmetic for Montgomery Curve25519.

pub type Fe = [u64; 5];

pub const FE_ZERO: Fe = [0; 5];
pub const FE_ONE: Fe = [1, 0, 0, 0, 0];

#[inline(always)]
pub fn fe_carry(f: &mut Fe) {
    for i in 0..4 {
        f[i + 1] += f[i] >> 51;
        f[i] &= (1u64 << 51) - 1;
    }
    let carry = f[4] >> 51;
    f[4] &= (1u64 << 51) - 1;
    f[0] += carry * 19;
    for i in 0..4 {
        f[i + 1] += f[i] >> 51;
        f[i] &= (1u64 << 51) - 1;
    }
}

#[inline(always)]
pub fn fe_add(a: &Fe, b: &Fe) -> Fe {
    let mut r = [0u64; 5];
    for i in 0..5 {
        r[i] = a[i] + b[i];
    }
    fe_carry(&mut r);
    r
}

#[inline(always)]
pub fn fe_sub(a: &Fe, b: &Fe) -> Fe {
    let mut r = [0u64; 5];
    let bias: [u64; 5] = [
        (1u64 << 51) * 2 - 38,
        (1u64 << 51) * 2 - 2,
        (1u64 << 51) * 2 - 2,
        (1u64 << 51) * 2 - 2,
        (1u64 << 51) * 2 - 2,
    ];
    for i in 0..5 {
        r[i] = a[i] + bias[i] - b[i];
    }
    fe_carry(&mut r);
    r
}

pub fn fe_mul(a: &Fe, b: &Fe) -> Fe {
    let mut t = [0u128; 5];

    for i in 0..5 {
        for j in 0..5 {
            let product = (a[i] as u128) * (b[j] as u128);
            let idx = i + j;
            if idx < 5 {
                t[idx] += product;
            } else {
                t[idx - 5] += product * 19;
            }
        }
    }

    let mut r = [0u64; 5];
    let mut carry = 0u128;
    for i in 0..5 {
        t[i] += carry;
        r[i] = (t[i] & ((1u128 << 51) - 1)) as u64;
        carry = t[i] >> 51;
    }
    r[0] += (carry * 19) as u64;
    fe_carry(&mut r);

    r
}

#[inline(always)]
pub fn fe_sq(a: &Fe) -> Fe {
    fe_mul(a, a)
}

pub fn fe_sq_n(a: &Fe, n: usize) -> Fe {
    let mut r = *a;
    for _ in 0..n {
        r = fe_sq(&r);
    }
    r
}

pub fn fe_invert(a: &Fe) -> Fe {
    let z2 = fe_sq(a);
    let z8 = fe_sq_n(&z2, 2);
    let z9 = fe_mul(a, &z8);
    let z11 = fe_mul(&z2, &z9);
    let z22 = fe_sq(&z11);
    let z_5_0 = fe_mul(&z9, &z22);
    let z_10_0 = fe_sq_n(&z_5_0, 5);
    let z_10_5 = fe_mul(&z_10_0, &z_5_0);
    let z_20_0 = fe_sq_n(&z_10_5, 10);
    let z_20_10 = fe_mul(&z_20_0, &z_10_5);
    let z_40_0 = fe_sq_n(&z_20_10, 20);
    let z_40_20 = fe_mul(&z_40_0, &z_20_10);
    let z_50_0 = fe_sq_n(&z_40_20, 10);
    let z_50_25 = fe_mul(&z_50_0, &z_10_5);
    let z_100_0 = fe_sq_n(&z_50_25, 50);
    let z_100_50 = fe_mul(&z_100_0, &z_50_25);
    let z_200_0 = fe_sq_n(&z_100_50, 100);
    let z_200_100 = fe_mul(&z_200_0, &z_100_50);
    let z_250_0 = fe_sq_n(&z_200_100, 50);
    let z_250_125 = fe_mul(&z_250_0, &z_50_25);
    let z_255_3 = fe_sq_n(&z_250_125, 5);
    fe_mul(&z_255_3, &z11)
}

pub fn fe_from_bytes(bytes: &[u8; 32]) -> Fe {
    let mut w0_bytes = [0u8; 8];
    let mut w1_bytes = [0u8; 8];
    let mut w2_bytes = [0u8; 8];
    let mut w3_bytes = [0u8; 8];
    w0_bytes.copy_from_slice(&bytes[0..8]);
    w1_bytes.copy_from_slice(&bytes[8..16]);
    w2_bytes.copy_from_slice(&bytes[16..24]);
    w3_bytes.copy_from_slice(&bytes[24..32]);

    let w0 = u64::from_le_bytes(w0_bytes);
    let w1 = u64::from_le_bytes(w1_bytes);
    let w2 = u64::from_le_bytes(w2_bytes);
    let w3 = u64::from_le_bytes(w3_bytes);

    const MASK51: u64 = (1u64 << 51) - 1;

    [
        w0 & MASK51,
        (w0 >> 51) | ((w1 << 13) & MASK51),
        (w1 >> 38) | ((w2 << 26) & MASK51),
        (w2 >> 25) | ((w3 << 39) & MASK51),
        (w3 >> 12) & MASK51,
    ]
}

pub fn fe_to_bytes(f: &Fe) -> [u8; 32] {
    let mut t = *f;
    fe_carry(&mut t);
    fe_carry(&mut t);
    fe_carry(&mut t);

    let mut q = (t[0] + 19) >> 51;
    q = (t[1] + q) >> 51;
    q = (t[2] + q) >> 51;
    q = (t[3] + q) >> 51;
    q = (t[4] + q) >> 51;

    t[0] += 19 * q;
    fe_carry(&mut t);

    let mut out = [0u8; 32];
    let w0 = t[0] | (t[1] << 51);
    let w1 = (t[1] >> 13) | (t[2] << 38);
    let w2 = (t[2] >> 26) | (t[3] << 25);
    let w3 = (t[3] >> 39) | (t[4] << 12);

    out[0..8].copy_from_slice(&w0.to_le_bytes());
    out[8..16].copy_from_slice(&w1.to_le_bytes());
    out[16..24].copy_from_slice(&w2.to_le_bytes());
    out[24..32].copy_from_slice(&w3.to_le_bytes());

    out[31] &= 0x7f;
    out
}

#[inline(always)]
pub fn cswap(a: &mut Fe, b: &mut Fe, swap: u64) {
    let mask = 0u64.wrapping_sub(swap);
    for i in 0..5 {
        let t = mask & (a[i] ^ b[i]);
        a[i] ^= t;
        b[i] ^= t;
    }
}
