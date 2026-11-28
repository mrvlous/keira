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

//! Unit tests for RFC 8446 TLS 1.3 handshake, key schedule and X.509 validation.

#[cfg(test)]
mod test {
    use super::super::handshake::{build_hkdf_label, decrypt_tls13_record, encrypt_tls13_record};
    use super::super::keyschedule::{
        compute_finished_verify_data, derive_application_secrets, derive_handshake_secrets,
        expand_traffic_key_iv, verify_constant_time_32,
    };
    use super::super::record::*;
    use super::super::x509::{is_ipv4_literal, match_dns_pattern, verify_certificate_payload};

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
        assert_eq!(buf[5], 0x01);
    }

    #[test]
    fn test_record_encryption_roundtrip() {
        let mut session = TlsSession::new();
        session.traffic_key = [0x42; 16];
        session.traffic_iv = [0x24; 12];

        let plaintext = b"Hello TLS 1.3 Secure World";
        let mut ciphertext = [0u8; 64];
        let (enc_len, tag) = session.encrypt_record(plaintext, &mut ciphertext);
        assert_eq!(enc_len, plaintext.len());

        let mut decrypted = [0u8; 64];
        let dec_res = session.decrypt_record_with_tag(&ciphertext[..enc_len], &tag, &mut decrypted);
        assert!(dec_res.is_ok());
        let dec_len = dec_res.unwrap();
        assert_eq!(dec_len, plaintext.len());
        assert_eq!(&decrypted[..dec_len], plaintext);

        let bad_tag = [0u8; 16];
        assert!(session
            .decrypt_record_with_tag(&ciphertext[..enc_len], &bad_tag, &mut decrypted)
            .is_err());
    }

    #[test]
    fn test_tls13_wire_aead_roundtrip_and_tamper_rejection() {
        let key = [0x11u8; 16];
        let iv = [0x22u8; 12];
        let mut tx_seq = 0u64;
        let mut rx_seq = 0u64;

        let msg = b"GET /api/status HTTP/1.1\r\nHost: keira\r\n\r\n";
        let mut plain_scratch = [0u8; 256];
        let mut wire = [0u8; 256];
        let wire_len = encrypt_tls13_record(
            &key,
            &iv,
            &mut tx_seq,
            TLS_CONTENT_APPLICATION_DATA,
            msg,
            &mut plain_scratch,
            &mut wire,
        )
        .expect("encrypt wire record");
        assert_eq!(tx_seq, 1);

        let mut hdr = [0u8; 5];
        hdr.copy_from_slice(&wire[..5]);
        let mut out_plain = [0u8; 256];
        let (ctype, plen) = decrypt_tls13_record(
            &key,
            &iv,
            &mut rx_seq,
            &hdr,
            &wire[5..wire_len],
            &mut out_plain,
        )
        .expect("decrypt wire record");
        assert_eq!(rx_seq, 1);
        assert_eq!(ctype, TLS_CONTENT_APPLICATION_DATA);
        assert_eq!(plen, msg.len());
        assert_eq!(&out_plain[..plen], msg);

        // Tamper one ciphertext byte and verify AEAD rejection
        wire[10] ^= 0xFF;
        let mut rx_seq_tamper = 0u64;
        assert!(decrypt_tls13_record(
            &key,
            &iv,
            &mut rx_seq_tamper,
            &hdr,
            &wire[5..wire_len],
            &mut out_plain,
        )
        .is_err());
    }

    #[test]
    fn test_rfc8446_key_schedule_and_finished_mac() {
        let shared = [0x5Au8; 32];
        let sh_hash = [0xA5u8; 32];
        let (hs_sec, c_hs, s_hs) = derive_handshake_secrets(&shared, &sh_hash);
        assert_ne!(c_hs, s_hs);
        let (c_key, c_iv) = expand_traffic_key_iv(&c_hs);
        let (s_key, s_iv) = expand_traffic_key_iv(&s_hs);
        assert_ne!(c_key, s_key);
        assert_ne!(c_iv, s_iv);

        let sf_hash = [0x3Cu8; 32];
        let (c_ap, s_ap) = derive_application_secrets(&hs_sec, &sf_hash);
        assert_ne!(c_ap, s_ap);

        let v1 = compute_finished_verify_data(&s_hs, &sf_hash);
        let v2 = compute_finished_verify_data(&s_hs, &sf_hash);
        assert!(verify_constant_time_32(&v1, &v2));
    }

    #[test]
    fn test_x509_dns_matching_and_ipv4_detection() {
        assert!(match_dns_pattern("api.ipify.org", "api.ipify.org"));
        assert!(match_dns_pattern("*.ipify.org", "api.ipify.org"));
        assert!(!match_dns_pattern("*.ipify.org", "sub.api.ipify.org"));
        assert!(!match_dns_pattern("example.com", "evil.com"));

        assert!(is_ipv4_literal("10.0.2.2"));
        assert!(!is_ipv4_literal("icanhazip.com"));

        // Minimal synthetic Certificate message containing SAN extension for *.keira.org
        let mut fake_cert = [0u8; 64];
        fake_cert[0] = 0; // certificate_request_context len = 0
                          // certificate_list length = 34
        fake_cert[1] = 0;
        fake_cert[2] = 0;
        fake_cert[3] = 34;
        // cert_data len = 29
        fake_cert[4] = 0;
        fake_cert[5] = 0;
        fake_cert[6] = 29;
        // X.509 DER sequence header + OID 2.5.29.17 + OCTET STRING + SEQ + DNSName "*.keira.org"
        let der: [u8; 29] = [
            0x30, 0x1B, 0x06, 0x03, 0x55, 0x1D, 0x11, 0x04, 0x14, 0x30, 0x12, 0x82, 0x0B, b'*',
            b'.', b'k', b'e', b'i', b'r', b'a', b'.', b'o', b'r', b'g', 0x00, 0x00, 0x00, 0x00,
            0x00,
        ];
        fake_cert[7..36].copy_from_slice(&der);
        // Extensions len = 0 at bytes 36..38

        let mut cn_out = [0u8; 128];
        let mut cn_len = 0;
        assert!(verify_certificate_payload(
            &fake_cert[..38],
            "net.keira.org",
            false,
            &mut cn_out,
            &mut cn_len
        )
        .is_ok());
        assert_eq!(&cn_out[..cn_len], b"*.keira.org");
        assert!(verify_certificate_payload(
            &fake_cert[..38],
            "other.org",
            false,
            &mut cn_out,
            &mut cn_len
        )
        .is_err());
        assert!(verify_certificate_payload(
            &fake_cert[..38],
            "other.org",
            true,
            &mut cn_out,
            &mut cn_len
        )
        .is_ok());
    }
}
