// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! TPM 2.0 sealed storage bound to PCR attestation policy masks.

use super::hardware::is_initialized;
use super::pcr::{init, log_event_internal, quote_pcrs, EV_ACTION};
use crate::cipher::gcm::{aes128_gcm_decrypt, aes128_gcm_encrypt};
use crate::hash::hmac::hmac_sha256;

pub const TPM_SEALED_MAGIC: [u8; 4] = *b"TPMS";
pub const TPM_MAX_SECRET_LEN: usize = 128;

/// Internal hardware security enclave root seed.
static TPM_SECRET_SEED: [u8; 32] = [
    0x54, 0x50, 0x4D, 0x32, 0x5F, 0x53, 0x45, 0x45, 0x44, 0x5F, 0x4B, 0x45, 0x49, 0x52, 0x41, 0x5F,
    0x52, 0x49, 0x4E, 0x47, 0x30, 0x5F, 0x53, 0x45, 0x43, 0x55, 0x52, 0x49, 0x54, 0x59, 0x30, 0x31,
];

/// TPM 2.0 Sealed Storage Data Blob (200 bytes, `repr(C)`).
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct TpmSealedBlob {
    pub magic: [u8; 4],
    pub pcr_mask: u32,
    pub expected_quote: [u8; 32],
    pub nonce: [u8; 12],
    pub data_len: u32,
    pub ciphertext: [u8; TPM_MAX_SECRET_LEN],
    pub auth_tag: [u8; 16],
}

/// Derive a 128-bit AES key from the TPM master seed and PCR quote using HKDF-SHA256.
pub fn derive_seal_key(quote: &[u8; 32]) -> [u8; 16] {
    let prk = hmac_sha256(&TPM_SECRET_SEED, quote);
    let info = b"TPM2_SEAL_STORAGE\x01";
    let okm = hmac_sha256(&prk, info);
    let mut key = [0u8; 16];
    key.copy_from_slice(&okm[..16]);
    key
}

/// Seal secret data bound to a specific PCR selection policy mask.
pub fn seal_secret(secret: &[u8], pcr_mask: u32) -> Result<TpmSealedBlob, &'static str> {
    if !is_initialized() {
        init();
    }
    if secret.is_empty() {
        return Err("Secret payload cannot be empty");
    }
    if secret.len() > TPM_MAX_SECRET_LEN {
        return Err("Secret payload exceeds maximum sealed capacity (128 bytes)");
    }

    let quote = quote_pcrs(pcr_mask);
    let key = derive_seal_key(&quote);

    let mut nonce = [0u8; 12];
    #[cfg(target_os = "none")]
    {
        let lo: u32;
        let hi: u32;
        unsafe {
            core::arch::asm!("rdtsc", out("eax") lo, out("edx") hi);
        }
        nonce[0..4].copy_from_slice(&lo.to_le_bytes());
        nonce[4..8].copy_from_slice(&hi.to_le_bytes());
        nonce[8..12].copy_from_slice(&(pcr_mask ^ 0xA5A5_5A5A).to_le_bytes());
    }
    #[cfg(not(target_os = "none"))]
    {
        nonce[0..4].copy_from_slice(&0x01020304u32.to_le_bytes());
        nonce[4..8].copy_from_slice(&0x05060708u32.to_le_bytes());
        nonce[8..12].copy_from_slice(&(pcr_mask ^ 0xA5A5_5A5A).to_le_bytes());
    }

    let mut ciphertext = [0u8; TPM_MAX_SECRET_LEN];
    let aad = b"KEIRA_TPM2_SEALED_ENCLAVE";
    let tag = aes128_gcm_encrypt(&key, &nonce, secret, aad, &mut ciphertext[..secret.len()]);

    unsafe {
        log_event_internal(
            (pcr_mask.trailing_zeros() as u8).min(23),
            EV_ACTION,
            quote,
            "TPM2_SEAL_SECRET",
        );
    }

    Ok(TpmSealedBlob {
        magic: TPM_SEALED_MAGIC,
        pcr_mask,
        expected_quote: quote,
        nonce,
        data_len: secret.len() as u32,
        ciphertext,
        auth_tag: tag,
    })
}

/// Unseal secret data, validating PCR policy attestation quote and GMAC authentication.
pub fn unseal_secret(blob: &TpmSealedBlob, out: &mut [u8]) -> Result<usize, &'static str> {
    if !is_initialized() {
        init();
    }
    if blob.magic != TPM_SEALED_MAGIC {
        return Err("Invalid sealed blob magic signature");
    }
    let data_len = blob.data_len as usize;
    if data_len > TPM_MAX_SECRET_LEN {
        return Err("Corrupt blob: payload length exceeds maximum capacity");
    }
    if out.len() < data_len {
        return Err("Destination buffer too small for unsealed secret");
    }

    let current_quote = quote_pcrs(blob.pcr_mask);
    if current_quote != blob.expected_quote {
        return Err("PCR policy integrity check failed: platform state mismatch");
    }

    let key = derive_seal_key(&blob.expected_quote);
    let aad = b"KEIRA_TPM2_SEALED_ENCLAVE";

    let ok = aes128_gcm_decrypt(
        &key,
        &blob.nonce,
        &blob.ciphertext[..data_len],
        aad,
        &blob.auth_tag,
        &mut out[..data_len],
    );

    if !ok {
        return Err("Sealed secret authentication failed: tamper detected");
    }

    unsafe {
        log_event_internal(
            (blob.pcr_mask.trailing_zeros() as u8).min(23),
            EV_ACTION,
            current_quote,
            "TPM2_UNSEAL_SECRET",
        );
    }

    Ok(data_len)
}
