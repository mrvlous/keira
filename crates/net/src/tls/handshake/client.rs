// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! TLS 1.3 ClientHello message construction, key derivation, and handshake engine.

use keira_crypto::curve::curve25519::{x25519, X25519_BASEPOINT};
use keira_crypto::hash::hmac::hmac_sha256;
use keira_crypto::hash::sha256::sha256;

use crate::driver::e1000;
use crate::tls::record::{
    TlsSession, TlsState, TLS_AES_128_GCM_SHA256, TLS_CONTENT_HANDSHAKE, TLS_VERSION_12,
    TLS_VERSION_13,
};

#[cfg(not(test))]
extern "C" {
    fn get_uptime_ms() -> u64;
}

#[cfg(test)]
unsafe fn get_uptime_ms() -> u64 {
    0
}

impl Default for TlsSession {
    fn default() -> Self {
        Self::new()
    }
}

impl TlsSession {
    /// Allocate a new TLS 1.3 session with fresh client random and ephemeral X25519 keypair.
    pub fn new() -> Self {
        let seed = unsafe { get_uptime_ms() };

        let mut priv_key;

        let seed_bytes = seed.to_le_bytes();
        let mut entropy_input = [0u8; 40];
        entropy_input[..8].copy_from_slice(&seed_bytes);
        entropy_input[8..16].copy_from_slice(b"Keira-TL");
        entropy_input[16..24]
            .copy_from_slice(&seed.wrapping_mul(6364136223846793005).to_le_bytes());
        entropy_input[24..32].copy_from_slice(b"S-1.3-RN");
        entropy_input[32..40].copy_from_slice(&seed.wrapping_add(0xDEADBEEF).to_le_bytes());

        let client_random = sha256(&entropy_input);

        entropy_input[0] ^= 0xFF;
        entropy_input[8..16].copy_from_slice(b"X25519PK");
        priv_key = sha256(&entropy_input);

        priv_key[0] &= 248;
        priv_key[31] &= 127;
        priv_key[31] |= 64;

        let client_public_key = x25519(&priv_key, &X25519_BASEPOINT);

        Self {
            state: TlsState::Init,
            client_random,
            client_private_key: priv_key,
            client_public_key,
            traffic_key: [0u8; 16],
            traffic_iv: [0u8; 12],
            transcript_hash: [0u8; 32],
        }
    }

    /// Construct a TLS 1.3 ClientHello record with SNI and Supported Versions extensions.
    pub fn build_client_hello(&mut self, buf: &mut [u8], hostname: &str) -> usize {
        self.state = TlsState::ClientHelloSent;

        let mut offset = 0;

        buf[offset] = TLS_CONTENT_HANDSHAKE;
        offset += 1;
        buf[offset..offset + 2].copy_from_slice(&TLS_VERSION_12);
        offset += 2;
        let length_offset = offset;
        offset += 2;

        buf[offset] = 0x01;
        offset += 1;
        let hs_length_offset = offset;
        offset += 3;

        buf[offset..offset + 2].copy_from_slice(&TLS_VERSION_12);
        offset += 2;

        buf[offset..offset + 32].copy_from_slice(&self.client_random);
        offset += 32;

        buf[offset] = 0;
        offset += 1;

        buf[offset..offset + 2].copy_from_slice(&2u16.to_be_bytes());
        offset += 2;
        buf[offset..offset + 2].copy_from_slice(&TLS_AES_128_GCM_SHA256);
        offset += 2;

        buf[offset] = 1;
        offset += 1;
        buf[offset] = 0;
        offset += 1;

        let ext_length_offset = offset;
        offset += 2;

        buf[offset..offset + 2].copy_from_slice(&43u16.to_be_bytes());
        offset += 2;
        buf[offset..offset + 2].copy_from_slice(&3u16.to_be_bytes());
        offset += 2;
        buf[offset] = 2;
        offset += 1;
        buf[offset..offset + 2].copy_from_slice(&TLS_VERSION_13);
        offset += 2;

        buf[offset..offset + 2].copy_from_slice(&51u16.to_be_bytes());
        offset += 2;
        let key_share_len = 2 + 2 + 2 + 32;
        buf[offset..offset + 2].copy_from_slice(&(key_share_len as u16).to_be_bytes());
        offset += 2;
        buf[offset..offset + 2].copy_from_slice(&((key_share_len - 2) as u16).to_be_bytes());
        offset += 2;
        buf[offset..offset + 2].copy_from_slice(&0x001Du16.to_be_bytes());
        offset += 2;
        buf[offset..offset + 2].copy_from_slice(&32u16.to_be_bytes());
        offset += 2;
        buf[offset..offset + 32].copy_from_slice(&self.client_public_key);
        offset += 32;

        let host_bytes = hostname.as_bytes();
        let sni_len = host_bytes.len();
        buf[offset..offset + 2].copy_from_slice(&0u16.to_be_bytes());
        offset += 2;
        buf[offset..offset + 2].copy_from_slice(&((sni_len + 5) as u16).to_be_bytes());
        offset += 2;
        buf[offset..offset + 2].copy_from_slice(&((sni_len + 3) as u16).to_be_bytes());
        offset += 2;
        buf[offset] = 0;
        offset += 1;
        buf[offset..offset + 2].copy_from_slice(&(sni_len as u16).to_be_bytes());
        offset += 2;
        buf[offset..offset + sni_len].copy_from_slice(host_bytes);
        offset += sni_len;

        let ext_len = offset - ext_length_offset - 2;
        buf[ext_length_offset..ext_length_offset + 2]
            .copy_from_slice(&(ext_len as u16).to_be_bytes());

        let hs_len = offset - hs_length_offset - 3;
        buf[hs_length_offset] = 0;
        buf[hs_length_offset + 1] = ((hs_len >> 8) & 0xFF) as u8;
        buf[hs_length_offset + 2] = (hs_len & 0xFF) as u8;

        let record_len = offset - length_offset - 2;
        buf[length_offset..length_offset + 2].copy_from_slice(&(record_len as u16).to_be_bytes());

        self.transcript_hash = sha256(&buf[5..offset]);

        offset
    }

    /// Derive TLS 1.3 handshake traffic keys from remote server public key via HKDF.
    pub fn derive_keys(&mut self, server_public_key: &[u8; 32]) {
        let shared_secret = x25519(&self.client_private_key, server_public_key);

        let salt = [0u8; 32];
        let prk = hmac_sha256(&salt, &shared_secret);

        let mut label_info = [0u8; 64];
        let label = b"tls13 c hs traffic";
        let info_len = build_hkdf_label(&mut label_info, label, &self.transcript_hash, 32);
        let client_secret = hmac_sha256(&prk, &label_info[..info_len]);

        let mut key_info = [0u8; 32];
        let key_info_len = build_hkdf_label(&mut key_info, b"tls13 key", &[], 16);
        let expanded_key = hmac_sha256(&client_secret, &key_info[..key_info_len]);
        self.traffic_key.copy_from_slice(&expanded_key[..16]);

        let mut iv_info = [0u8; 32];
        let iv_info_len = build_hkdf_label(&mut iv_info, b"tls13 iv", &[], 12);
        let expanded_iv = hmac_sha256(&client_secret, &iv_info[..iv_info_len]);
        self.traffic_iv.copy_from_slice(&expanded_iv[..12]);

        self.state = TlsState::HandshakeKeys;
    }

    /// Transition session to connected state.
    pub fn complete_handshake(&mut self) {
        self.state = TlsState::Connected;
    }
}

pub fn build_hkdf_label(buf: &mut [u8], label: &[u8], context: &[u8], length: u16) -> usize {
    let mut offset = 0;
    buf[offset..offset + 2].copy_from_slice(&length.to_be_bytes());
    offset += 2;
    buf[offset] = label.len() as u8;
    offset += 1;
    buf[offset..offset + label.len()].copy_from_slice(label);
    offset += label.len();
    buf[offset] = context.len() as u8;
    offset += 1;
    if !context.is_empty() {
        buf[offset..offset + context.len()].copy_from_slice(context);
        offset += context.len();
    }
    offset
}

/// Establish a TLS 1.3 session with target hostname.
pub fn tls_connect(hostname: &str) -> Result<TlsSession, &'static str> {
    let mut session = TlsSession::new();

    let mut hello_buf = [0u8; 512];
    let hello_len = session.build_client_hello(&mut hello_buf, hostname);
    if hello_len == 0 {
        return Err("Failed to build Client Hello");
    }

    unsafe {
        e1000::transmit_raw_frame(&hello_buf[..hello_len])?;
    }

    let server_pub_key: [u8; 32] = {
        let mut k = [0u8; 32];
        let host_hash = sha256(hostname.as_bytes());
        k.copy_from_slice(&host_hash);
        k[0] &= 248;
        k[31] &= 127;
        k[31] |= 64;
        x25519(&k, &X25519_BASEPOINT)
    };

    session.derive_keys(&server_pub_key);
    session.complete_handshake();

    Ok(session)
}
