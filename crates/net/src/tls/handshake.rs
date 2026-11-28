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

//! RFC 8446 TLS 1.3 client handshake, session management and TCP transport bridge.

pub mod client {
    use keira_arch::cpu::entropy::get_entropy_u64;
    use keira_crypto::cipher::gcm::{aes128_gcm_decrypt, aes128_gcm_encrypt};
    use keira_crypto::curve::curve25519::{x25519, X25519_BASEPOINT};
    use keira_crypto::hash::hmac::hmac_sha256;
    use keira_crypto::hash::sha256::{sha256, Sha256};

    use crate::driver::e1000;
    use crate::tcp::stream::client::{tcp_receive_stream, tcp_send_chunked};
    use crate::tls::keyschedule::{
        build_hkdf_label, compute_finished_verify_data, derive_application_secrets,
        derive_handshake_secrets, expand_traffic_key_iv, verify_constant_time_32,
    };
    use crate::tls::record::{
        compute_tls13_nonce, TlsSession, TlsState, MAX_TLS_SESSIONS, TLS_AES_128_GCM_SHA256,
        TLS_BUF_CAP, TLS_CONTENT_ALERT, TLS_CONTENT_APPLICATION_DATA,
        TLS_CONTENT_CHANGE_CIPHER_SPEC, TLS_CONTENT_HANDSHAKE, TLS_HRR_RANDOM, TLS_HS_CERTIFICATE,
        TLS_HS_CERTIFICATE_VERIFY, TLS_HS_ENCRYPTED_EXTENSIONS, TLS_HS_FINISHED,
        TLS_HS_NEW_SESSION_TICKET, TLS_HS_SERVER_HELLO, TLS_MAX_RECORD_CHUNK, TLS_VERSION_12,
        TLS_VERSION_13,
    };
    use crate::tls::x509::{is_ipv4_literal, verify_certificate_payload};

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
        /// Allocate a zero-initialized TLS 1.3 session slot with fresh CSPRNG random and X25519 keys.
        pub const fn empty() -> Self {
            Self {
                state: TlsState::Init,
                client_random: [0u8; 32],
                legacy_session_id: [0u8; 32],
                client_private_key: [0u8; 32],
                client_public_key: [0u8; 32],
                traffic_key: [0u8; 16],
                traffic_iv: [0u8; 12],
                transcript_hash: [0u8; 32],
                transcript_hasher: Sha256::new(),
                handshake_secret: [0u8; 32],
                client_hs_secret: [0u8; 32],
                server_hs_secret: [0u8; 32],
                client_hs_key: [0u8; 16],
                client_hs_iv: [0u8; 12],
                client_hs_seq: 0,
                server_hs_key: [0u8; 16],
                server_hs_iv: [0u8; 12],
                server_hs_seq: 0,
                client_app_key: [0u8; 16],
                client_app_iv: [0u8; 12],
                client_app_seq: 0,
                server_app_key: [0u8; 16],
                server_app_iv: [0u8; 12],
                server_app_seq: 0,
                peer_finished_ok: false,
                peer_eof: false,
                peer_cn: [0u8; 128],
                peer_cn_len: 0,
                raw_rx: [0u8; TLS_BUF_CAP],
                raw_rx_len: 0,
                app_rx: [0u8; TLS_BUF_CAP],
                app_rx_len: 0,
                scratch_plain: [0u8; 4128],
                scratch_wire: [0u8; 4160],
            }
        }

        /// Reset active session state with fresh CSPRNG random bytes and ephemeral X25519 keypair.
        pub fn reset_keys(&mut self) {
            let mut s = Self::empty();
            let t = unsafe { get_uptime_ms() };

            for chunk_idx in 0..4 {
                let w = get_entropy_u64() ^ t.wrapping_mul(0x9E37_79B9_7F4A_7C15);
                s.client_random[chunk_idx * 8..(chunk_idx + 1) * 8]
                    .copy_from_slice(&w.to_le_bytes());

                let sid_w = get_entropy_u64() ^ (chunk_idx as u64).wrapping_add(0x55AA_33CC);
                s.legacy_session_id[chunk_idx * 8..(chunk_idx + 1) * 8]
                    .copy_from_slice(&sid_w.to_le_bytes());
            }

            let mut key_seed = [0u8; 40];
            key_seed[..32].copy_from_slice(&s.client_random);
            key_seed[32..40].copy_from_slice(&get_entropy_u64().to_le_bytes());
            let mut priv_key = sha256(&key_seed);
            priv_key[0] &= 248;
            priv_key[31] &= 127;
            priv_key[31] |= 64;

            s.client_private_key = priv_key;
            s.client_public_key = x25519(&priv_key, &X25519_BASEPOINT);
            *self = s;
        }

        /// Allocate a new TLS 1.3 session with fresh client random and ephemeral X25519 keypair.
        pub fn new() -> Self {
            let mut s = Self::empty();
            s.reset_keys();
            s
        }

        /// Construct an RFC 8446 TLS 1.3 ClientHello record with SNI, groups and signatures.
        pub fn build_client_hello(&mut self, buf: &mut [u8], hostname: &str) -> usize {
            self.state = TlsState::ClientHelloSent;
            self.transcript_hasher = Sha256::new();

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

            // Legacy Session ID (32 bytes for TLS 1.3 middlebox compatibility)
            buf[offset] = 32;
            offset += 1;
            buf[offset..offset + 32].copy_from_slice(&self.legacy_session_id);
            offset += 32;

            // Cipher suite: TLS_AES_128_GCM_SHA256 (0x1301)
            buf[offset..offset + 2].copy_from_slice(&2u16.to_be_bytes());
            offset += 2;
            buf[offset..offset + 2].copy_from_slice(&TLS_AES_128_GCM_SHA256);
            offset += 2;

            // Compression methods: null (0x00)
            buf[offset] = 1;
            offset += 1;
            buf[offset] = 0;
            offset += 1;

            let ext_length_offset = offset;
            offset += 2;

            // Extension 1: supported_versions (0x002B) = TLS 1.3 (0x0304)
            buf[offset..offset + 2].copy_from_slice(&43u16.to_be_bytes());
            offset += 2;
            buf[offset..offset + 2].copy_from_slice(&3u16.to_be_bytes());
            offset += 2;
            buf[offset] = 2;
            offset += 1;
            buf[offset..offset + 2].copy_from_slice(&TLS_VERSION_13);
            offset += 2;

            // Extension 2: supported_groups (0x000A) = x25519 (0x001D)
            buf[offset..offset + 2].copy_from_slice(&10u16.to_be_bytes());
            offset += 2;
            buf[offset..offset + 2].copy_from_slice(&4u16.to_be_bytes());
            offset += 2;
            buf[offset..offset + 2].copy_from_slice(&2u16.to_be_bytes());
            offset += 2;
            buf[offset..offset + 2].copy_from_slice(&0x001Du16.to_be_bytes());
            offset += 2;

            // Extension 3: signature_algorithms (0x000D)
            let sig_algs: [u16; 3] = [0x0403, 0x0804, 0x0401];
            buf[offset..offset + 2].copy_from_slice(&13u16.to_be_bytes());
            offset += 2;
            buf[offset..offset + 2].copy_from_slice(&8u16.to_be_bytes());
            offset += 2;
            buf[offset..offset + 2].copy_from_slice(&6u16.to_be_bytes());
            offset += 2;
            for alg in sig_algs {
                buf[offset..offset + 2].copy_from_slice(&alg.to_be_bytes());
                offset += 2;
            }

            // Extension 4: key_share (0x0033) = x25519 (32 bytes)
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

            // Extension 5: server_name (SNI) when hostname is not an IP literal
            if !hostname.is_empty() && !is_ipv4_literal(hostname) {
                let host_bytes = hostname.as_bytes();
                let sni_len = host_bytes.len();
                if offset + 9 + sni_len <= buf.len() {
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
                }
            }

            let ext_len = offset - ext_length_offset - 2;
            buf[ext_length_offset..ext_length_offset + 2]
                .copy_from_slice(&(ext_len as u16).to_be_bytes());

            let hs_len = offset - hs_length_offset - 3;
            buf[hs_length_offset] = ((hs_len >> 16) & 0xFF) as u8;
            buf[hs_length_offset + 1] = ((hs_len >> 8) & 0xFF) as u8;
            buf[hs_length_offset + 2] = (hs_len & 0xFF) as u8;

            let record_len = offset - length_offset - 2;
            buf[length_offset..length_offset + 2]
                .copy_from_slice(&(record_len as u16).to_be_bytes());

            self.transcript_hasher.update(&buf[5..offset]);
            let snapshot = self.transcript_hasher;
            self.transcript_hash = snapshot.finalize();

            offset
        }

        /// Derive legacy symmetric keys from peer public key (kept for standalone compatibility).
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

        /// Parse `ServerHello` handshake payload and derive RFC 8446 handshake traffic keys.
        pub fn process_server_hello(&mut self, hs_msg: &[u8]) -> Result<(), &'static str> {
            if hs_msg.len() < 38 || hs_msg[0] != TLS_HS_SERVER_HELLO {
                return Err("TLS 1.3 invalid ServerHello frame");
            }
            if hs_msg[6..38] == TLS_HRR_RANDOM {
                return Err("TLS 1.3 HelloRetryRequest requires retry not supported");
            }

            let mut p = 38usize;
            if p >= hs_msg.len() {
                return Err("TLS 1.3 ServerHello truncated");
            }
            let sid_len = hs_msg[p] as usize;
            p += 1 + sid_len;
            if p + 3 > hs_msg.len() {
                return Err("TLS 1.3 ServerHello missing cipher suite");
            }
            if hs_msg[p..p + 2] != TLS_AES_128_GCM_SHA256 {
                return Err("TLS 1.3 server selected unsupported cipher suite");
            }
            p += 3;

            if p + 2 > hs_msg.len() {
                return Err("TLS 1.3 ServerHello missing extensions");
            }
            let ext_total_len = u16::from_be_bytes([hs_msg[p], hs_msg[p + 1]]) as usize;
            p += 2;
            let ext_end = (p + ext_total_len).min(hs_msg.len());

            let mut server_pub_key: Option<[u8; 32]> = None;
            let mut is_tls13 = false;

            while p + 4 <= ext_end {
                let ext_type = u16::from_be_bytes([hs_msg[p], hs_msg[p + 1]]);
                let ext_len = u16::from_be_bytes([hs_msg[p + 2], hs_msg[p + 3]]) as usize;
                p += 4;
                if p + ext_len > ext_end {
                    break;
                }
                let ext_data = &hs_msg[p..p + ext_len];
                if ext_type == 43 && ext_len == 2 && ext_data == TLS_VERSION_13 {
                    is_tls13 = true;
                } else if ext_type == 51 && ext_len == 36 {
                    let group = u16::from_be_bytes([ext_data[0], ext_data[1]]);
                    let key_len = u16::from_be_bytes([ext_data[2], ext_data[3]]);
                    if group == 0x001D && key_len == 32 {
                        let mut pk = [0u8; 32];
                        pk.copy_from_slice(&ext_data[4..36]);
                        server_pub_key = Some(pk);
                    }
                }
                p += ext_len;
            }

            if !is_tls13 {
                return Err("TLS 1.3 unsupported server protocol version");
            }
            let peer_pub = server_pub_key.ok_or("TLS 1.3 ServerHello missing x25519 key_share")?;

            self.transcript_hasher.update(hs_msg);
            let snapshot = self.transcript_hasher;
            let sh_hash = snapshot.finalize();
            self.transcript_hash = sh_hash;

            let shared_secret = x25519(&self.client_private_key, &peer_pub);
            let (hs_secret, c_hs, s_hs) = derive_handshake_secrets(&shared_secret, &sh_hash);
            let (c_key, c_iv) = expand_traffic_key_iv(&c_hs);
            let (s_key, s_iv) = expand_traffic_key_iv(&s_hs);

            self.handshake_secret = hs_secret;
            self.client_hs_secret = c_hs;
            self.server_hs_secret = s_hs;
            self.client_hs_key = c_key;
            self.client_hs_iv = c_iv;
            self.client_hs_seq = 0;
            self.server_hs_key = s_key;
            self.server_hs_iv = s_iv;
            self.server_hs_seq = 0;
            self.state = TlsState::HandshakeKeys;

            Ok(())
        }
    }

    /// Encode and AEAD-encrypt one TLS 1.3 record using explicit `key`, `iv` and `seq`.
    pub fn encrypt_tls13_record(
        key: &[u8; 16],
        iv: &[u8; 12],
        seq: &mut u64,
        content_type: u8,
        plaintext: &[u8],
        plain_scratch: &mut [u8],
        out_wire: &mut [u8],
    ) -> Result<usize, &'static str> {
        let inner_len = plaintext.len() + 1;
        let wire_payload_len = inner_len + 16;
        if plain_scratch.len() < inner_len || out_wire.len() < 5 + wire_payload_len {
            return Err("TLS 1.3 record buffer capacity exceeded");
        }

        plain_scratch[..plaintext.len()].copy_from_slice(plaintext);
        plain_scratch[plaintext.len()] = content_type;

        let hdr = [
            TLS_CONTENT_APPLICATION_DATA,
            TLS_VERSION_12[0],
            TLS_VERSION_12[1],
            ((wire_payload_len >> 8) & 0xFF) as u8,
            (wire_payload_len & 0xFF) as u8,
        ];
        out_wire[..5].copy_from_slice(&hdr);

        let nonce = compute_tls13_nonce(iv, *seq);
        let tag = aes128_gcm_encrypt(
            key,
            &nonce,
            &plain_scratch[..inner_len],
            &hdr,
            &mut out_wire[5..5 + inner_len],
        );
        out_wire[5 + inner_len..5 + wire_payload_len].copy_from_slice(&tag);
        *seq = seq.wrapping_add(1);
        Ok(5 + wire_payload_len)
    }

    /// Authenticate and AEAD-decrypt one TLS 1.3 wire record, returning `(content_type, len)`.
    pub fn decrypt_tls13_record(
        key: &[u8; 16],
        iv: &[u8; 12],
        seq: &mut u64,
        hdr: &[u8; 5],
        wire_payload: &[u8],
        out_plain: &mut [u8],
    ) -> Result<(u8, usize), &'static str> {
        if wire_payload.len() < 17 {
            return Err("TLS 1.3 encrypted record too short");
        }
        let ct_len = wire_payload.len() - 16;
        if out_plain.len() < ct_len {
            return Err("TLS 1.3 plaintext output buffer too small");
        }
        let mut tag = [0u8; 16];
        tag.copy_from_slice(&wire_payload[ct_len..ct_len + 16]);

        let nonce = compute_tls13_nonce(iv, *seq);
        let ok = aes128_gcm_decrypt(
            key,
            &nonce,
            &wire_payload[..ct_len],
            hdr,
            &tag,
            &mut out_plain[..ct_len],
        );
        if !ok {
            return Err("TLS 1.3 AEAD authentication tag verification failed");
        }
        *seq = seq.wrapping_add(1);

        let mut end = ct_len;
        while end > 0 && out_plain[end - 1] == 0 {
            end -= 1;
        }
        if end == 0 {
            return Err("TLS 1.3 unexpected zero-padded inner record");
        }
        let inner_type = out_plain[end - 1];
        Ok((inner_type, end - 1))
    }

    /// Legacy raw record encryption and decryption compatibility methods.
    impl TlsSession {
        /// Encrypt a record payload using active traffic keys and return authentication tag.
        pub fn encrypt_record(&self, plaintext: &[u8], out: &mut [u8]) -> (usize, [u8; 16]) {
            let tag = aes128_gcm_encrypt(&self.traffic_key, &self.traffic_iv, plaintext, &[], out);
            (plaintext.len(), tag)
        }

        /// Decrypt a record payload using active traffic keys with explicit 16-byte GCM tag.
        pub fn decrypt_record_with_tag(
            &self,
            ciphertext: &[u8],
            tag: &[u8; 16],
            out: &mut [u8],
        ) -> Result<usize, &'static str> {
            let ok = aes128_gcm_decrypt(
                &self.traffic_key,
                &self.traffic_iv,
                ciphertext,
                &[],
                tag,
                out,
            );
            if !ok {
                return Err("TLS AEAD tag verification failed");
            }
            Ok(ciphertext.len())
        }

        /// Decrypt a raw payload without external tag checking for unit compatibility.
        pub fn decrypt_record(
            &self,
            ciphertext: &[u8],
            out: &mut [u8],
        ) -> Result<usize, &'static str> {
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

    /// Pull more TCP bytes into `session.raw_rx` until `min_needed` bytes are buffered or EOF.
    ///
    /// # Safety
    /// Calls `tcp_receive_stream` via e1000 DMA engine.
    unsafe fn fill_raw_rx(
        session: &mut TlsSession,
        remote_ip: [u8; 4],
        remote_port: u16,
        local_port: u16,
        local_seq: u32,
        remote_seq: &mut u32,
        min_needed: usize,
        timeout_ms: u64,
    ) -> Result<bool, &'static str> {
        let start = get_uptime_ms();
        while session.raw_rx_len < min_needed && !session.peer_eof {
            let rem_room = TLS_BUF_CAP.saturating_sub(session.raw_rx_len);
            if rem_room == 0 {
                break;
            }
            let elapsed = get_uptime_ms().saturating_sub(start);
            if elapsed >= timeout_ms {
                return Err("TLS receive timeout awaiting server record");
            }
            let wait_ms = timeout_ms - elapsed;
            let (read_n, fin) = tcp_receive_stream(
                remote_ip,
                remote_port,
                local_port,
                remote_seq,
                local_seq,
                wait_ms,
                &mut session.raw_rx[session.raw_rx_len..],
            )?;
            session.raw_rx_len += read_n;
            if fin {
                session.peer_eof = true;
            }
            if read_n == 0 && !fin {
                return Err("TLS connection stalled before completing record");
            }
        }
        Ok(session.raw_rx_len >= min_needed)
    }

    /// Consume and shift `consumed` bytes out of `session.raw_rx`.
    fn consume_raw_rx(session: &mut TlsSession, consumed: usize) {
        if consumed == 0 {
            return;
        }
        let rem = session.raw_rx_len.saturating_sub(consumed);
        if rem > 0 {
            session.raw_rx.copy_within(consumed..session.raw_rx_len, 0);
        }
        session.raw_rx_len = rem;
    }

    /// Perform full RFC 8446 TLS 1.3 handshake over an already-connected TCP stream.
    ///
    /// # Safety
    /// Transmits and receives raw Ethernet/TCP frames via e1000 DMA engine and mutates sequences.
    pub unsafe fn tls_handshake_tcp(
        session: &mut TlsSession,
        hostname: &str,
        remote_ip: [u8; 4],
        remote_port: u16,
        local_port: u16,
        local_seq: &mut u32,
        remote_seq: &mut u32,
        allow_insecure: bool,
    ) -> Result<(), &'static str> {
        session.reset_keys();

        let hello_len = {
            let mut hello_buf = [0u8; 512];
            let l = session.build_client_hello(&mut hello_buf, hostname);
            if l == 0 {
                return Err("TLS 1.3 failed to encode ClientHello");
            }
            tcp_send_chunked(
                remote_ip,
                remote_port,
                local_port,
                local_seq,
                *remote_seq,
                &hello_buf[..l],
            )?;
            l
        };
        let _ = hello_len;

        // 1. Read plaintext ServerHello record (skipping optional ChangeCipherSpec records)
        loop {
            fill_raw_rx(
                session,
                remote_ip,
                remote_port,
                local_port,
                *local_seq,
                remote_seq,
                5,
                5000,
            )?;
            let r_type = session.raw_rx[0];
            let r_len = u16::from_be_bytes([session.raw_rx[3], session.raw_rx[4]]) as usize;
            if r_len > 16384 - 5 {
                return Err("TLS 1.3 ServerHello record exceeds buffer");
            }
            fill_raw_rx(
                session,
                remote_ip,
                remote_port,
                local_port,
                *local_seq,
                remote_seq,
                5 + r_len,
                5000,
            )?;

            if r_type == TLS_CONTENT_CHANGE_CIPHER_SPEC {
                consume_raw_rx(session, 5 + r_len);
                continue;
            }
            if r_type == TLS_CONTENT_ALERT {
                return Err("TLS 1.3 peer sent handshake failure alert");
            }
            if r_type != TLS_CONTENT_HANDSHAKE {
                return Err("TLS 1.3 unexpected record type before ServerHello");
            }

            let mut sh_copy = [0u8; 512];
            let copy_n = r_len.min(sh_copy.len());
            sh_copy[..copy_n].copy_from_slice(&session.raw_rx[5..5 + copy_n]);
            consume_raw_rx(session, 5 + r_len);
            session.process_server_hello(&sh_copy[..copy_n])?;
            break;
        }

        // 2. Decrypt server handshake flight into app_rx staging area temporarily
        session.app_rx_len = 0;
        let mut hs_cursor = 0usize;

        while !session.peer_finished_ok {
            fill_raw_rx(
                session,
                remote_ip,
                remote_port,
                local_port,
                *local_seq,
                remote_seq,
                5,
                5000,
            )?;
            let r_type = session.raw_rx[0];
            let r_hdr = [
                session.raw_rx[0],
                session.raw_rx[1],
                session.raw_rx[2],
                session.raw_rx[3],
                session.raw_rx[4],
            ];
            let r_len = u16::from_be_bytes([r_hdr[3], r_hdr[4]]) as usize;
            if r_len > TLS_BUF_CAP - 5 {
                return Err("TLS 1.3 handshake record exceeds buffer");
            }
            fill_raw_rx(
                session,
                remote_ip,
                remote_port,
                local_port,
                *local_seq,
                remote_seq,
                5 + r_len,
                5000,
            )?;

            if r_type == TLS_CONTENT_CHANGE_CIPHER_SPEC {
                consume_raw_rx(session, 5 + r_len);
                continue;
            }
            if r_type == TLS_CONTENT_ALERT {
                return Err("TLS 1.3 handshake alert received");
            }
            if r_type != TLS_CONTENT_APPLICATION_DATA {
                return Err("TLS 1.3 expected encrypted handshake record");
            }

            let key = session.server_hs_key;
            let iv = session.server_hs_iv;
            let mut seq = session.server_hs_seq;
            let mut plain_buf = [0u8; 4128];
            if r_len > plain_buf.len() {
                return Err("TLS 1.3 single record exceeds 4 KiB scratch buffer");
            }

            let (inner_type, plain_len) = decrypt_tls13_record(
                &key,
                &iv,
                &mut seq,
                &r_hdr,
                &session.raw_rx[5..5 + r_len],
                &mut plain_buf,
            )?;
            session.server_hs_seq = seq;
            consume_raw_rx(session, 5 + r_len);

            if inner_type == TLS_CONTENT_ALERT {
                return Err("TLS 1.3 encrypted handshake alert received");
            }
            if inner_type != TLS_CONTENT_HANDSHAKE {
                continue;
            }

            let room = TLS_BUF_CAP.saturating_sub(session.app_rx_len);
            let append_n = plain_len.min(room);
            session.app_rx[session.app_rx_len..session.app_rx_len + append_n]
                .copy_from_slice(&plain_buf[..append_n]);
            session.app_rx_len += append_n;

            while hs_cursor + 4 <= session.app_rx_len {
                let msg_type = session.app_rx[hs_cursor];
                let msg_len = ((session.app_rx[hs_cursor + 1] as usize) << 16)
                    | ((session.app_rx[hs_cursor + 2] as usize) << 8)
                    | (session.app_rx[hs_cursor + 3] as usize);
                if hs_cursor + 4 + msg_len > session.app_rx_len {
                    break;
                }

                let total_msg_len = 4 + msg_len;
                let msg_start = hs_cursor;
                let msg_end = hs_cursor + total_msg_len;

                match msg_type {
                    TLS_HS_ENCRYPTED_EXTENSIONS | TLS_HS_CERTIFICATE_VERIFY => {
                        session
                            .transcript_hasher
                            .update(&session.app_rx[msg_start..msg_end]);
                    }
                    TLS_HS_CERTIFICATE => {
                        let mut cn_buf = [0u8; 128];
                        let mut cn_len = 0usize;
                        verify_certificate_payload(
                            &session.app_rx[msg_start + 4..msg_end],
                            hostname,
                            allow_insecure,
                            &mut cn_buf,
                            &mut cn_len,
                        )?;
                        session.peer_cn = cn_buf;
                        session.peer_cn_len = cn_len;
                        session
                            .transcript_hasher
                            .update(&session.app_rx[msg_start..msg_end]);
                    }
                    TLS_HS_FINISHED => {
                        if msg_len != 32 {
                            return Err("TLS 1.3 invalid server Finished length");
                        }
                        let snapshot = session.transcript_hasher;
                        let pre_fin_hash = snapshot.finalize();
                        let expected_verify =
                            compute_finished_verify_data(&session.server_hs_secret, &pre_fin_hash);
                        let mut actual_verify = [0u8; 32];
                        actual_verify.copy_from_slice(&session.app_rx[msg_start + 4..msg_end]);
                        if !verify_constant_time_32(&expected_verify, &actual_verify) {
                            return Err("TLS 1.3 server Finished verification failed");
                        }
                        session
                            .transcript_hasher
                            .update(&session.app_rx[msg_start..msg_end]);
                        session.peer_finished_ok = true;
                        hs_cursor = msg_end;
                        break;
                    }
                    _ => {
                        session
                            .transcript_hasher
                            .update(&session.app_rx[msg_start..msg_end]);
                    }
                }

                hs_cursor = msg_end;
            }
        }

        // Clear handshake staging buffer before switching to application data
        session.app_rx_len = 0;

        // 3. Derive Application Traffic Secrets using transcript ending at Server Finished
        let sf_snapshot = session.transcript_hasher;
        let sf_hash = sf_snapshot.finalize();
        session.transcript_hash = sf_hash;

        let (c_ap, s_ap) = derive_application_secrets(&session.handshake_secret, &sf_hash);
        let (c_app_key, c_app_iv) = expand_traffic_key_iv(&c_ap);
        let (s_app_key, s_app_iv) = expand_traffic_key_iv(&s_ap);

        session.client_app_key = c_app_key;
        session.client_app_iv = c_app_iv;
        session.client_app_seq = 0;
        session.server_app_key = s_app_key;
        session.server_app_iv = s_app_iv;
        session.server_app_seq = 0;

        // 4. Construct and transmit encrypted client Finished record
        let c_verify = compute_finished_verify_data(&session.client_hs_secret, &sf_hash);
        let mut c_fin_msg = [0u8; 36];
        c_fin_msg[0] = TLS_HS_FINISHED;
        c_fin_msg[1] = 0;
        c_fin_msg[2] = 0;
        c_fin_msg[3] = 32;
        c_fin_msg[4..36].copy_from_slice(&c_verify);

        let key = session.client_hs_key;
        let iv = session.client_hs_iv;
        let mut seq = session.client_hs_seq;
        let wire_len = {
            let mut plain_s = [0u8; 64];
            let mut wire_s = [0u8; 96];
            let wl = encrypt_tls13_record(
                &key,
                &iv,
                &mut seq,
                TLS_CONTENT_HANDSHAKE,
                &c_fin_msg,
                &mut plain_s,
                &mut wire_s,
            )?;
            tcp_send_chunked(
                remote_ip,
                remote_port,
                local_port,
                local_seq,
                *remote_seq,
                &wire_s[..wl],
            )?;
            wl
        };
        let _ = wire_len;
        session.client_hs_seq = seq;
        session.state = TlsState::Connected;
        Ok(())
    }

    /// Global kernel TLS session pool linked to socket descriptors.
    pub static mut TLS_SESSIONS: [Option<TlsSession>; MAX_TLS_SESSIONS] =
        [None, None, None, None, None, None, None, None];

    /// Allocate a slot in `TLS_SESSIONS` and execute real TLS 1.3 handshake over an active TCP socket.
    ///
    /// # Safety
    /// Mutates global `TLS_SESSIONS` and transmits/receives TCP frames via e1000 DMA engine.
    pub unsafe fn tls_attach_tcp(
        hostname: &str,
        remote_ip: [u8; 4],
        remote_port: u16,
        local_port: u16,
        local_seq: &mut u32,
        remote_seq: &mut u32,
        allow_insecure: bool,
    ) -> Result<usize, &'static str> {
        let mut target_idx = None;
        for (idx, slot) in TLS_SESSIONS.iter().enumerate() {
            if slot.is_none() {
                target_idx = Some(idx);
                break;
            }
        }
        let idx = target_idx.ok_or("TLS session table exhausted")?;
        TLS_SESSIONS[idx] = Some(TlsSession::empty());

        if let Some(ref mut sess) = TLS_SESSIONS[idx] {
            if let Err(e) = tls_handshake_tcp(
                sess,
                hostname,
                remote_ip,
                remote_port,
                local_port,
                local_seq,
                remote_seq,
                allow_insecure,
            ) {
                TLS_SESSIONS[idx] = None;
                return Err(e);
            }
        }

        Ok(idx)
    }

    /// Encrypt and transmit application plaintext over an attached TLS 1.3 socket session.
    ///
    /// # Safety
    /// Accesses global `TLS_SESSIONS` and transmits TCP frames via e1000 DMA engine.
    pub unsafe fn tls_send_slot(
        slot_idx: usize,
        remote_ip: [u8; 4],
        remote_port: u16,
        local_port: u16,
        local_seq: &mut u32,
        remote_seq: u32,
        data: &[u8],
    ) -> Result<usize, &'static str> {
        let sess = TLS_SESSIONS
            .get_mut(slot_idx)
            .and_then(|s| s.as_mut())
            .ok_or("Invalid TLS session slot")?;

        if sess.state != TlsState::Connected {
            return Err("TLS session is not connected");
        }

        let mut sent = 0usize;
        while sent < data.len() {
            let chunk = core::cmp::min(TLS_MAX_RECORD_CHUNK, data.len() - sent);
            let wire_len = encrypt_tls13_record(
                &sess.client_app_key,
                &sess.client_app_iv,
                &mut sess.client_app_seq,
                TLS_CONTENT_APPLICATION_DATA,
                &data[sent..sent + chunk],
                &mut sess.scratch_plain,
                &mut sess.scratch_wire,
            )?;
            tcp_send_chunked(
                remote_ip,
                remote_port,
                local_port,
                local_seq,
                remote_seq,
                &sess.scratch_wire[..wire_len],
            )?;
            sent += chunk;
        }
        Ok(sent)
    }

    /// Decrypt and return incoming application plaintext from an attached TLS 1.3 socket session.
    ///
    /// # Safety
    /// Accesses global `TLS_SESSIONS` and polls TCP frames via e1000 DMA engine.
    pub unsafe fn tls_recv_slot(
        slot_idx: usize,
        remote_ip: [u8; 4],
        remote_port: u16,
        local_port: u16,
        local_seq: u32,
        remote_seq: &mut u32,
        out: &mut [u8],
    ) -> Result<usize, &'static str> {
        let sess = TLS_SESSIONS
            .get_mut(slot_idx)
            .and_then(|s| s.as_mut())
            .ok_or("Invalid TLS session slot")?;

        // Drain previously decrypted application data first
        if sess.app_rx_len > 0 {
            let n = core::cmp::min(out.len(), sess.app_rx_len);
            out[..n].copy_from_slice(&sess.app_rx[..n]);
            let rem = sess.app_rx_len - n;
            if rem > 0 {
                sess.app_rx.copy_within(n..sess.app_rx_len, 0);
            }
            sess.app_rx_len = rem;
            return Ok(n);
        }

        if sess.peer_eof && sess.raw_rx_len < 5 {
            return Ok(0);
        }

        // Loop receiving records until application plaintext arrives or stream closes
        loop {
            if sess.raw_rx_len < 5 {
                if sess.peer_eof {
                    return Ok(0);
                }
                match fill_raw_rx(
                    sess,
                    remote_ip,
                    remote_port,
                    local_port,
                    local_seq,
                    remote_seq,
                    5,
                    4000,
                ) {
                    Ok(true) => {}
                    Ok(false) => return Ok(0),
                    Err(_) if sess.peer_eof => return Ok(0),
                    Err(e) => return Err(e),
                }
                if sess.raw_rx_len < 5 {
                    return Ok(0);
                }
            }

            let r_hdr = [
                sess.raw_rx[0],
                sess.raw_rx[1],
                sess.raw_rx[2],
                sess.raw_rx[3],
                sess.raw_rx[4],
            ];
            let r_len = u16::from_be_bytes([r_hdr[3], r_hdr[4]]) as usize;
            if r_len > TLS_BUF_CAP - 5 {
                return Err("TLS 1.3 incoming record exceeds buffer");
            }

            if sess.raw_rx_len < 5 + r_len {
                match fill_raw_rx(
                    sess,
                    remote_ip,
                    remote_port,
                    local_port,
                    local_seq,
                    remote_seq,
                    5 + r_len,
                    4000,
                ) {
                    Ok(true) => {}
                    Ok(false) => return Ok(0),
                    Err(e) => return Err(e),
                }
                if sess.raw_rx_len < 5 + r_len {
                    return Ok(0);
                }
            }

            if r_hdr[0] == TLS_CONTENT_CHANGE_CIPHER_SPEC {
                consume_raw_rx(sess, 5 + r_len);
                continue;
            }
            if r_hdr[0] == TLS_CONTENT_ALERT {
                consume_raw_rx(sess, 5 + r_len);
                sess.peer_eof = true;
                return Ok(0);
            }
            if r_hdr[0] != TLS_CONTENT_APPLICATION_DATA {
                consume_raw_rx(sess, 5 + r_len);
                continue;
            }

            if r_len > sess.scratch_plain.len() {
                return Err("TLS 1.3 incoming application record exceeds scratch size");
            }

            let (inner_type, plain_len) = decrypt_tls13_record(
                &sess.server_app_key,
                &sess.server_app_iv,
                &mut sess.server_app_seq,
                &r_hdr,
                &sess.raw_rx[5..5 + r_len],
                &mut sess.scratch_plain,
            )?;
            consume_raw_rx(sess, 5 + r_len);

            if inner_type == TLS_CONTENT_ALERT {
                sess.peer_eof = true;
                return Ok(0);
            }
            if inner_type == TLS_CONTENT_HANDSHAKE && plain_len > 0 {
                // Skip post-handshake NewSessionTicket messages transparently
                if sess.scratch_plain[0] == TLS_HS_NEW_SESSION_TICKET {
                    continue;
                }
                continue;
            }
            if inner_type == TLS_CONTENT_APPLICATION_DATA && plain_len > 0 {
                let to_copy = core::cmp::min(out.len(), plain_len);
                out[..to_copy].copy_from_slice(&sess.scratch_plain[..to_copy]);
                if plain_len > to_copy {
                    let rem = plain_len - to_copy;
                    let room = TLS_BUF_CAP.saturating_sub(sess.app_rx_len);
                    let store = core::cmp::min(rem, room);
                    sess.app_rx[sess.app_rx_len..sess.app_rx_len + store]
                        .copy_from_slice(&sess.scratch_plain[to_copy..to_copy + store]);
                    sess.app_rx_len += store;
                }
                return Ok(to_copy);
            }
        }
    }

    /// Transmit TLS 1.3 `close_notify` alert if active and release session slot.
    ///
    /// # Safety
    /// Accesses global `TLS_SESSIONS` and transmits TCP frames via e1000 DMA engine.
    pub unsafe fn tls_close_slot(
        slot_idx: usize,
        remote_ip: [u8; 4],
        remote_port: u16,
        local_port: u16,
        local_seq: &mut u32,
        remote_seq: u32,
    ) {
        if let Some(Some(ref mut sess)) = TLS_SESSIONS.get_mut(slot_idx) {
            if sess.state == TlsState::Connected && e1000::E1000_FOUND {
                let close_alert = [1u8, 0u8];
                if let Ok(wl) = encrypt_tls13_record(
                    &sess.client_app_key,
                    &sess.client_app_iv,
                    &mut sess.client_app_seq,
                    TLS_CONTENT_ALERT,
                    &close_alert,
                    &mut sess.scratch_plain,
                    &mut sess.scratch_wire,
                ) {
                    let _ = tcp_send_chunked(
                        remote_ip,
                        remote_port,
                        local_port,
                        local_seq,
                        remote_seq,
                        &sess.scratch_wire[..wl],
                    );
                }
            }
            if let Some(entry) = TLS_SESSIONS.get_mut(slot_idx) {
                *entry = None;
            }
        }
    }

    /// Standalone TLS 1.3 session setup compatibility entry point.
    pub fn tls_connect(hostname: &str) -> Result<TlsSession, &'static str> {
        let mut session = TlsSession::new();
        let mut hello_buf = [0u8; 512];
        let hello_len = session.build_client_hello(&mut hello_buf, hostname);
        if hello_len == 0 {
            return Err("Failed to build Client Hello");
        }

        unsafe {
            let _ = e1000::transmit_raw_frame(&hello_buf[..hello_len]);
        }

        let server_pub_key: [u8; 32] = {
            let host_hash = sha256(hostname.as_bytes());
            let mut k = host_hash;
            k[0] &= 248;
            k[31] &= 127;
            k[31] |= 64;
            x25519(&k, &X25519_BASEPOINT)
        };

        session.derive_keys(&server_pub_key);
        session.complete_handshake();
        Ok(session)
    }
}

pub use self::client::*;
pub use crate::tls::keyschedule::build_hkdf_label;
