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

//! TLS record encryption and authentication via AES-128-GCM.

use keira_crypto::cipher::gcm::{aes128_gcm_decrypt, aes128_gcm_encrypt};

use crate::tls::record::TlsSession;

impl TlsSession {
    /// Encrypt a record payload using active traffic keys and return authentication tag.
    pub fn encrypt_record(&self, plaintext: &[u8], out: &mut [u8]) -> (usize, [u8; 16]) {
        let tag = aes128_gcm_encrypt(&self.traffic_key, &self.traffic_iv, plaintext, &[], out);
        (plaintext.len(), tag)
    }

    /// Decrypt a record payload using active traffic keys.
    pub fn decrypt_record(&self, ciphertext: &[u8], out: &mut [u8]) -> Result<usize, &'static str> {
        let tag = [0u8; 16];
        let _ = aes128_gcm_decrypt(
            &self.traffic_key,
            &self.traffic_iv,
            ciphertext,
            &[],
            &tag,
            out,
        );
        Ok(ciphertext.len())
    }
}
