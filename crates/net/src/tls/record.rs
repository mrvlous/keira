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

//! Transport Layer Security (TLS 1.3) cryptographic record and session state machine.

pub mod types {
    use keira_crypto::hash::sha256::Sha256;

    pub const TLS_CONTENT_CHANGE_CIPHER_SPEC: u8 = 20;
    pub const TLS_CONTENT_ALERT: u8 = 21;
    pub const TLS_CONTENT_HANDSHAKE: u8 = 22;
    pub const TLS_CONTENT_APPLICATION_DATA: u8 = 23;

    pub const TLS_VERSION_12: [u8; 2] = [0x03, 0x03];
    pub const TLS_VERSION_13: [u8; 2] = [0x03, 0x04];
    pub const TLS_AES_128_GCM_SHA256: [u8; 2] = [0x13, 0x01];

    pub const TLS_HS_CLIENT_HELLO: u8 = 1;
    pub const TLS_HS_SERVER_HELLO: u8 = 2;
    pub const TLS_HS_NEW_SESSION_TICKET: u8 = 4;
    pub const TLS_HS_ENCRYPTED_EXTENSIONS: u8 = 8;
    pub const TLS_HS_CERTIFICATE: u8 = 11;
    pub const TLS_HS_CERTIFICATE_VERIFY: u8 = 15;
    pub const TLS_HS_FINISHED: u8 = 20;

    /// HelloRetryRequest fixed random identifier per RFC 8446 section 4.1.3.
    pub const TLS_HRR_RANDOM: [u8; 32] = [
        0xCF, 0x21, 0xAD, 0x74, 0xE5, 0x9A, 0x61, 0x11, 0xBE, 0x1D, 0x8C, 0x02, 0x1E, 0x65, 0xB8,
        0x91, 0xC2, 0xA2, 0x11, 0x16, 0x7A, 0xBB, 0x8C, 0x5E, 0x07, 0x9E, 0x09, 0xE2, 0xC8, 0xA8,
        0x33, 0x9C,
    ];

    /// Size of TLS 1.3 staging buffers attached per active TLS session.
    pub const TLS_BUF_CAP: usize = 16384;

    /// Maximum single record plaintext chunk size sent over TLS 1.3.
    pub const TLS_MAX_RECORD_CHUNK: usize = 2048;

    /// Maximum concurrent kernel-side TLS sessions across socket descriptors.
    pub const MAX_TLS_SESSIONS: usize = 8;

    /// TLS 1.3 protocol handshake state machine stages.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum TlsState {
        Init,
        ClientHelloSent,
        HandshakeKeys,
        Connected,
        Closed,
    }

    /// Active TLS 1.3 cryptographic session state and streaming buffers.
    pub struct TlsSession {
        pub state: TlsState,
        pub client_random: [u8; 32],
        pub legacy_session_id: [u8; 32],
        pub client_private_key: [u8; 32],
        pub client_public_key: [u8; 32],
        pub traffic_key: [u8; 16],
        pub traffic_iv: [u8; 12],
        pub transcript_hash: [u8; 32],
        pub transcript_hasher: Sha256,
        pub handshake_secret: [u8; 32],
        pub client_hs_secret: [u8; 32],
        pub server_hs_secret: [u8; 32],
        pub client_hs_key: [u8; 16],
        pub client_hs_iv: [u8; 12],
        pub client_hs_seq: u64,
        pub server_hs_key: [u8; 16],
        pub server_hs_iv: [u8; 12],
        pub server_hs_seq: u64,
        pub client_app_key: [u8; 16],
        pub client_app_iv: [u8; 12],
        pub client_app_seq: u64,
        pub server_app_key: [u8; 16],
        pub server_app_iv: [u8; 12],
        pub server_app_seq: u64,
        pub peer_finished_ok: bool,
        pub peer_eof: bool,
        pub peer_cn: [u8; 128],
        pub peer_cn_len: usize,
        pub raw_rx: [u8; TLS_BUF_CAP],
        pub raw_rx_len: usize,
        pub app_rx: [u8; TLS_BUF_CAP],
        pub app_rx_len: usize,
        pub scratch_plain: [u8; 4128],
        pub scratch_wire: [u8; 4160],
    }

    /// Compute the per-record AEAD nonce by XORing big-endian sequence into `iv`.
    pub fn compute_tls13_nonce(iv: &[u8; 12], seq: u64) -> [u8; 12] {
        let mut nonce = *iv;
        let seq_be = seq.to_be_bytes();
        for i in 0..8 {
            nonce[4 + i] ^= seq_be[i];
        }
        nonce
    }
}

pub use self::types::*;
