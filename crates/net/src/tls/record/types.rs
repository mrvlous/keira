// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Transport Layer Security (TLS) 1.3 record protocol constants and session state.

pub const TLS_CONTENT_HANDSHAKE: u8 = 22;
pub const TLS_CONTENT_APPLICATION_DATA: u8 = 23;
pub const TLS_VERSION_12: [u8; 2] = [0x03, 0x03];
pub const TLS_VERSION_13: [u8; 2] = [0x03, 0x04];
pub const TLS_AES_128_GCM_SHA256: [u8; 2] = [0x13, 0x01];

/// TLS 1.3 protocol handshake state machine stages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TlsState {
    Init,
    ClientHelloSent,
    HandshakeKeys,
    Connected,
    Closed,
}

/// Active TLS 1.3 cryptographic session state.
pub struct TlsSession {
    pub state: TlsState,
    pub client_random: [u8; 32],
    pub client_private_key: [u8; 32],
    pub client_public_key: [u8; 32],
    pub traffic_key: [u8; 16],
    pub traffic_iv: [u8; 12],
    pub transcript_hash: [u8; 32],
}
