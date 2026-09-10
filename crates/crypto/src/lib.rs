// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

#![no_std]

//! Pure freestanding cryptographic primitives (SHA-256, HMAC, AES-128-GCM, Curve25519, TPM 2.0).

pub mod cipher;
pub mod curve;
pub mod hash;
pub mod tpm;

pub use cipher::*;
pub use curve::*;
pub use hash::*;
pub use tpm::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tpm2_pcr_init_and_extend() {
        tpm::init();
        assert!(tpm::is_initialized());

        let pcr0 = tpm::read_pcr(0).expect("PCR 0 should be readable");
        assert_ne!(pcr0, [0u8; 32]);

        let extended = tpm::extend_pcr(10, b"measurement_payload", "TEST_MEASURE")
            .expect("Extend should succeed");
        assert_ne!(extended, [0u8; 32]);

        let pcr10 = tpm::read_pcr(10).expect("PCR 10 should be readable");
        assert_eq!(extended, pcr10);

        let quote = tpm::quote_pcrs(0b0001);
        assert_ne!(quote, [0u8; 32]);
    }
}
