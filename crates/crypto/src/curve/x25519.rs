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

//! Curve25519 and X25519 Diffie-Hellman key exchange subsystem.

pub mod field {
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
}

pub mod scalar {
    use super::field::{
        cswap, fe_add, fe_from_bytes, fe_invert, fe_mul, fe_sq, fe_sub, fe_to_bytes, FE_ONE,
        FE_ZERO,
    };

    pub const X25519_BASEPOINT: [u8; 32] = {
        let mut bp = [0u8; 32];
        bp[0] = 9;
        bp
    };

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
}

pub use self::field::*;
pub use self::scalar::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_x25519_key_exchange_vectors() {
        let alice_priv: [u8; 32] = [
            0x77, 0x07, 0x6d, 0x0a, 0x73, 0x18, 0xa5, 0x7d, 0x3c, 0x16, 0xc1, 0x72, 0x51, 0x52,
            0x53, 0x02, 0xf5, 0x0e, 0xc3, 0x4d, 0x03, 0xf8, 0xa7, 0x28, 0x36, 0x5b, 0x6f, 0x44,
            0xd2, 0x36, 0x01, 0x4e,
        ];
        let expected_alice_pub: [u8; 32] = [
            0x0b, 0xce, 0x39, 0x0c, 0xe5, 0x3e, 0x5b, 0xde, 0x17, 0xf4, 0xed, 0x48, 0x69, 0x52,
            0x0e, 0x19, 0x01, 0x41, 0xe1, 0xe7, 0x31, 0x68, 0x36, 0x86, 0x0f, 0x0f, 0xdc, 0x12,
            0x82, 0xaf, 0xdd, 0x54,
        ];

        let bob_priv: [u8; 32] = [
            0x5d, 0xab, 0x08, 0x7e, 0x62, 0x4a, 0x8a, 0x4b, 0x79, 0xe1, 0x7f, 0x8b, 0x83, 0x80,
            0x0e, 0xe6, 0x6f, 0x3b, 0xb1, 0x29, 0x26, 0x18, 0xb6, 0xfd, 0x1c, 0x2f, 0x8b, 0x27,
            0xff, 0x88, 0xe0, 0xeb,
        ];
        let expected_bob_pub: [u8; 32] = [
            0xde, 0x9e, 0xdb, 0x7d, 0x7b, 0x7d, 0xc1, 0xb4, 0xd3, 0x5b, 0x61, 0xc2, 0xec, 0xe4,
            0x35, 0x37, 0x3f, 0x83, 0x43, 0xc8, 0x5b, 0x78, 0x67, 0x4d, 0xad, 0xfc, 0x7e, 0x14,
            0x6f, 0x88, 0x2b, 0x4f,
        ];

        let alice_pub = x25519(&alice_priv, &X25519_BASEPOINT);
        assert_eq!(alice_pub, expected_alice_pub);

        let bob_pub = x25519(&bob_priv, &X25519_BASEPOINT);
        assert_eq!(bob_pub, expected_bob_pub);

        let shared_alice = x25519(&alice_priv, &bob_pub);
        let shared_bob = x25519(&bob_priv, &alice_pub);

        assert_eq!(shared_alice, shared_bob);
        let expected_shared: [u8; 32] = [
            0x0d, 0xcd, 0x59, 0x36, 0xbc, 0xd6, 0x2f, 0x03, 0x4d, 0xe4, 0xf0, 0x5e, 0x13, 0x97,
            0xb3, 0x4e, 0x42, 0x4e, 0x35, 0x95, 0xc8, 0x1c, 0xf7, 0x29, 0xf8, 0x2f, 0x9a, 0xa7,
            0x53, 0x3a, 0x32, 0x5b,
        ];
        assert_eq!(shared_alice, expected_shared);
    }
}
