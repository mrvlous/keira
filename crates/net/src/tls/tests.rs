// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Unit tests for TLS 1.3 protocol and cryptographic record handling.

#[cfg(test)]
mod test {
    use super::super::handshake::build_hkdf_label;
    use super::super::record::*;

    #[test]
    fn test_tls_session_creation() {
        let session = TlsSession::new();
        assert_eq!(session.state, TlsState::Init);
        assert_ne!(session.client_random, [0u8; 32]);
        assert_ne!(session.client_public_key, [0u8; 32]);
    }

    #[test]
    fn test_hkdf_label_generation() {
        let mut buf = [0u8; 64];
        let label = b"test";
        let context = b"ctx";
        let len = build_hkdf_label(&mut buf, label, context, 16);
        assert_eq!(len, 2 + 1 + 4 + 1 + 3);
        assert_eq!(&buf[0..2], &16u16.to_be_bytes());
        assert_eq!(buf[2], 4);
        assert_eq!(&buf[3..7], b"test");
        assert_eq!(buf[7], 3);
        assert_eq!(&buf[8..11], b"ctx");
    }

    #[test]
    fn test_client_hello_formatting() {
        let mut session = TlsSession::new();
        let mut buf = [0u8; 512];
        let len = session.build_client_hello(&mut buf, "example.com");
        assert!(len > 50);
        assert_eq!(session.state, TlsState::ClientHelloSent);

        assert_eq!(buf[0], TLS_CONTENT_HANDSHAKE);
        assert_eq!(buf[1], TLS_VERSION_12[0]);
        assert_eq!(buf[2], TLS_VERSION_12[1]);
        assert_eq!(buf[5], 0x01); // Handshake Type: Client Hello
    }

    #[test]
    fn test_record_encryption_roundtrip() {
        let mut session = TlsSession::new();
        session.traffic_key = [0x42; 16];
        session.traffic_iv = [0x24; 12];

        let plaintext = b"Hello TLS 1.3 Secure World";
        let mut ciphertext = [0u8; 64];
        let (enc_len, _tag) = session.encrypt_record(plaintext, &mut ciphertext);
        assert_eq!(enc_len, plaintext.len());

        let mut decrypted = [0u8; 64];
        let dec_res = session.decrypt_record(&ciphertext[..enc_len], &mut decrypted);
        assert!(dec_res.is_ok());
        let dec_len = dec_res.unwrap();
        assert_eq!(dec_len, plaintext.len());
        assert_eq!(&decrypted[..dec_len], plaintext);
    }
}
