// SPDX-License-Identifier: GPL-2.0-only
//
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

//! RFC 8446 TLS 1.3 HKDF-SHA256 key schedule and verify data derivation.

use keira_crypto::hash::hmac::hmac_sha256;
use keira_crypto::hash::sha256::sha256;

/// Encode a TLS 1.3 `HkdfLabel` structure into `buf` and return encoded byte length.
pub fn build_hkdf_label(buf: &mut [u8], label: &[u8], context: &[u8], length: u16) -> usize {
    let mut offset = 0;
    if buf.len() < 4 + label.len() + context.len() {
        return 0;
    }
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

/// Compute `HKDF-Extract(salt, ikm)` using HMAC-SHA256.
pub fn hkdf_extract(salt: &[u8], ikm: &[u8]) -> [u8; 32] {
    let zero_salt = [0u8; 32];
    let effective_salt = if salt.is_empty() { &zero_salt } else { salt };
    hmac_sha256(effective_salt, ikm)
}

/// Compute `HKDF-Expand-Label(secret, label, context, out.len())` up to 32 bytes.
///
/// Callers supply the full wire label including `"tls13 "` prefix.
pub fn hkdf_expand_label(secret: &[u8; 32], label: &[u8], context: &[u8], out: &mut [u8]) {
    let out_len = out.len().min(32);
    let mut info = [0u8; 128];
    let info_len = build_hkdf_label(&mut info, label, context, out_len as u16);
    if info_len == 0 || info_len >= info.len() {
        return;
    }
    info[info_len] = 0x01;
    let okm = hmac_sha256(secret, &info[..info_len + 1]);
    out[..out_len].copy_from_slice(&okm[..out_len]);
}

/// Compute `Derive-Secret(secret, label, transcript_hash)` per RFC 8446 section 7.1.
pub fn derive_secret(secret: &[u8; 32], label: &[u8], transcript_hash: &[u8; 32]) -> [u8; 32] {
    let mut out = [0u8; 32];
    hkdf_expand_label(secret, label, transcript_hash, &mut out);
    out
}

/// Expand a traffic secret into paired 16-byte AES-128-GCM key and 12-byte IV.
pub fn expand_traffic_key_iv(secret: &[u8; 32]) -> ([u8; 16], [u8; 12]) {
    let mut key = [0u8; 16];
    let mut iv = [0u8; 12];
    hkdf_expand_label(secret, b"tls13 key", &[], &mut key);
    hkdf_expand_label(secret, b"tls13 iv", &[], &mut iv);
    (key, iv)
}

/// Compute RFC 8446 `Finished` verify_data from `base_key` and `transcript_hash`.
pub fn compute_finished_verify_data(base_key: &[u8; 32], transcript_hash: &[u8; 32]) -> [u8; 32] {
    let mut finished_key = [0u8; 32];
    hkdf_expand_label(base_key, b"tls13 finished", &[], &mut finished_key);
    hmac_sha256(&finished_key, transcript_hash)
}

/// Perform constant-time equality comparison across two 32-byte digests.
pub fn verify_constant_time_32(a: &[u8; 32], b: &[u8; 32]) -> bool {
    let mut diff = 0u8;
    for i in 0..32 {
        diff |= a[i] ^ b[i];
    }
    diff == 0
}

/// Derive full TLS 1.3 handshake secrets and key/IV pairs from X25519 shared secret.
pub fn derive_handshake_secrets(
    shared_secret: &[u8; 32],
    transcript_sh_hash: &[u8; 32],
) -> ([u8; 32], [u8; 32], [u8; 32]) {
    let zeros = [0u8; 32];
    let early_secret = hkdf_extract(&zeros, &zeros);
    let empty_hash = sha256(&[]);
    let derived = derive_secret(&early_secret, b"tls13 derived", &empty_hash);
    let handshake_secret = hkdf_extract(&derived, shared_secret);
    let client_hs = derive_secret(&handshake_secret, b"tls13 c hs traffic", transcript_sh_hash);
    let server_hs = derive_secret(&handshake_secret, b"tls13 s hs traffic", transcript_sh_hash);
    (handshake_secret, client_hs, server_hs)
}

/// Derive TLS 1.3 application traffic secrets from `handshake_secret` and server Finished hash.
pub fn derive_application_secrets(
    handshake_secret: &[u8; 32],
    transcript_sf_hash: &[u8; 32],
) -> ([u8; 32], [u8; 32]) {
    let zeros = [0u8; 32];
    let empty_hash = sha256(&[]);
    let derived = derive_secret(handshake_secret, b"tls13 derived", &empty_hash);
    let master_secret = hkdf_extract(&derived, &zeros);
    let client_ap = derive_secret(&master_secret, b"tls13 c ap traffic", transcript_sf_hash);
    let server_ap = derive_secret(&master_secret, b"tls13 s ap traffic", transcript_sf_hash);
    (client_ap, server_ap)
}
