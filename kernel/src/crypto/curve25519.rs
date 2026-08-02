// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Keira Kernel: Curve25519 Elliptic Curve Diffie-Hellman (RFC 7748) Subsystem
//!
//! Provides bare-metal implementation of X25519 scalar multiplication for ECDH key exchange.

type Fe = [u64; 5];

const FE_ZERO: Fe = [0; 5];
const FE_ONE: Fe = [1, 0, 0, 0, 0];

fn fe_carry(f: &mut Fe) {
    for i in 0..4 {
        f[i + 1] += f[i] >> 51;
        f[i] &= (1u64 << 51) - 1;
    }
    let carry = f[4] >> 51;
    f[4] &= (1u64 << 51) - 1;
    f[0] += carry * 19;
}

fn fe_add(a: &Fe, b: &Fe) -> Fe {
    let mut r = [0u64; 5];
    for i in 0..5 {
        r[i] = a[i] + b[i];
    }
    fe_carry(&mut r);
    r
}

fn fe_sub(a: &Fe, b: &Fe) -> Fe {
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

fn fe_mul(a: &Fe, b: &Fe) -> Fe {
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
    for i in 0..5 {
        r[i] = t[i] as u64;
        if i < 4 {
            t[i + 1] += t[i] >> 51;
        }
        r[i] &= (1u64 << 51) - 1;
    }

    let carry = r[4] >> 51;
    r[4] &= (1u64 << 51) - 1;
    r[0] += carry * 19;
    fe_carry(&mut r);

    r
}

fn fe_sq(a: &Fe) -> Fe {
    fe_mul(a, a)
}

fn fe_sq_n(a: &Fe, n: usize) -> Fe {
    let mut r = *a;
    for _ in 0..n {
        r = fe_sq(&r);
    }
    r
}

fn fe_invert(a: &Fe) -> Fe {
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

fn fe_from_bytes(bytes: &[u8; 32]) -> Fe {
    let mut r = FE_ZERO;
    r[0] = u64::from_le_bytes([
        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], 0, 0,
    ]) & ((1u64 << 51) - 1);
    r[1] = (u64::from_le_bytes([
        bytes[6], bytes[7], bytes[8], bytes[9], bytes[10], bytes[11], bytes[12], 0,
    ]) >> 3)
        & ((1u64 << 51) - 1);
    r[2] = (u64::from_le_bytes([
        bytes[12], bytes[13], bytes[14], bytes[15], bytes[16], bytes[17], bytes[18], bytes[19],
    ]) >> 6)
        & ((1u64 << 51) - 1);
    r[3] = (u64::from_le_bytes([
        bytes[19], bytes[20], bytes[21], bytes[22], bytes[23], bytes[24], bytes[25], 0,
    ]) >> 1)
        & ((1u64 << 51) - 1);
    r[4] = (u64::from_le_bytes([
        bytes[25], bytes[26], bytes[27], bytes[28], bytes[29], bytes[30], bytes[31], 0,
    ]) >> 4)
        & ((1u64 << 51) - 1);
    r
}

fn fe_to_bytes(f: &Fe) -> [u8; 32] {
    let mut t = *f;
    fe_carry(&mut t);
    fe_carry(&mut t);

    let mut out = [0u8; 32];
    let mut acc: u128 = 0;
    let mut bit_pos = 0usize;

    for i in 0..5 {
        acc |= (t[i] as u128) << bit_pos;
        bit_pos += 51;
    }

    for i in 0..32 {
        out[i] = (acc >> (i * 8)) as u8;
    }

    out[31] &= 0x7f;
    out
}

pub fn x25519(scalar: &[u8; 32], point: &[u8; 32]) -> [u8; 32] {
    let mut k = *scalar;
    k[0] &= 248;
    k[31] &= 127;
    k[31] |= 64;

    let u = fe_from_bytes(point);

    let x_1 = u;
    let mut x_2 = FE_ONE;
    let mut z_2 = FE_ZERO;
    let mut x_3 = u;
    let mut z_3 = FE_ONE;
    let mut swap: u64 = 0;

    for t in (0..255).rev() {
        let k_t = ((k[t / 8] >> (t & 7)) & 1) as u64;
        swap ^= k_t;

        cswap(&mut x_2, &mut x_3, swap);
        cswap(&mut z_2, &mut z_3, swap);
        swap = k_t;

        let a = fe_add(&x_2, &z_2);
        let aa = fe_sq(&a);
        let b = fe_sub(&x_2, &z_2);
        let bb = fe_sq(&b);
        let e = fe_sub(&aa, &bb);
        let c = fe_add(&x_3, &z_3);
        let d = fe_sub(&x_3, &z_3);
        let da = fe_mul(&d, &a);
        let cb = fe_mul(&c, &b);
        x_3 = fe_sq(&fe_add(&da, &cb));
        z_3 = fe_mul(&x_1, &fe_sq(&fe_sub(&da, &cb)));
        x_2 = fe_mul(&aa, &bb);
        let a24 = [121665u64, 0, 0, 0, 0];
        z_2 = fe_mul(&e, &fe_add(&aa, &fe_mul(&a24, &e)));
    }

    cswap(&mut x_2, &mut x_3, swap);
    cswap(&mut z_2, &mut z_3, swap);

    let result = fe_mul(&x_2, &fe_invert(&z_2));
    fe_to_bytes(&result)
}

fn cswap(a: &mut Fe, b: &mut Fe, swap: u64) {
    let mask = 0u64.wrapping_sub(swap);
    for i in 0..5 {
        let t = mask & (a[i] ^ b[i]);
        a[i] ^= t;
        b[i] ^= t;
    }
}

pub const X25519_BASEPOINT: [u8; 32] = {
    let mut bp = [0u8; 32];
    bp[0] = 9;
    bp
};
