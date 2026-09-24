// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! X25519 (RFC 7748) scalar multiplication and Diffie-Hellman function.

use super::field::{
    cswap, fe_add, fe_from_bytes, fe_invert, fe_mul, fe_sq, fe_sub, fe_to_bytes, FE_ONE, FE_ZERO,
};

/// Canonical basepoint generator $u = 9$ for Montgomery Curve25519.
pub const X25519_BASEPOINT: [u8; 32] = {
    let mut bp = [0u8; 32];
    bp[0] = 9;
    bp
};

/// Compute X25519 scalar multiplication: point * scalar (RFC 7748 Section 5).
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
