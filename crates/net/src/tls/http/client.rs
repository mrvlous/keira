// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Encrypted HTTPS client primitives running over native TLS 1.3.

use crate::driver::e1000::E1000_FOUND;
use crate::tls::handshake::tls_connect;

/// Fetch an HTTPS resource over native TLS 1.3 encapsulated network stack.
pub unsafe fn fetch_https(
    hostname: &str,
    target_path: &str,
) -> Result<([u8; 512], usize), &'static str> {
    if !E1000_FOUND {
        return Err("Network card offline");
    }

    let target_ip = crate::dns::resolver::resolve_domain(hostname).unwrap_or([10, 0, 2, 2]);
    let session = tls_connect(hostname)?;

    let mut req_buf = [0u8; 512];
    let mut req_len = 0;
    let req_str = b"GET ";
    req_buf[req_len..req_len + req_str.len()].copy_from_slice(req_str);
    req_len += req_str.len();

    let p_bytes = target_path.as_bytes();
    let to_copy_p = core::cmp::min(p_bytes.len(), 256);
    req_buf[req_len..req_len + to_copy_p].copy_from_slice(&p_bytes[..to_copy_p]);
    req_len += to_copy_p;

    let host_prefix = b" HTTP/1.1\r\nHost: ";
    req_buf[req_len..req_len + host_prefix.len()].copy_from_slice(host_prefix);
    req_len += host_prefix.len();

    let h_bytes = hostname.as_bytes();
    let to_copy_h = core::cmp::min(h_bytes.len(), 64);
    req_buf[req_len..req_len + to_copy_h].copy_from_slice(&h_bytes[..to_copy_h]);
    req_len += to_copy_h;

    let ua_prefix = b"\r\nUser-Agent: ";
    req_buf[req_len..req_len + ua_prefix.len()].copy_from_slice(ua_prefix);
    req_len += ua_prefix.len();

    let ua_bytes = crate::HTTP_USER_AGENT.as_bytes();
    req_buf[req_len..req_len + ua_bytes.len()].copy_from_slice(ua_bytes);
    req_len += ua_bytes.len();

    let req_end = b"\r\nConnection: close\r\n\r\n";
    req_buf[req_len..req_len + req_end.len()].copy_from_slice(req_end);
    req_len += req_end.len();

    let mut enc_buf = [0u8; 512];
    let (enc_len, _tag) = session.encrypt_record(&req_buf[..req_len], &mut enc_buf);

    match crate::tcp::stream::tcp_send_and_receive(target_ip, 443, &enc_buf[..enc_len]) {
        Ok((payload, len)) => {
            let mut out_buf = [0u8; 512];
            if len >= 5 && (payload[0] == 0x17 || payload[0] == 0x16) {
                let record_len = u16::from_be_bytes([payload[3], payload[4]]) as usize;
                let record_data = if 5 + record_len <= len {
                    &payload[5..5 + record_len]
                } else {
                    &payload[5..len]
                };
                let res_len = session
                    .decrypt_record(record_data, &mut out_buf)
                    .unwrap_or(0);
                if res_len > 0 {
                    return Ok((out_buf, res_len));
                }
            }
            Ok((payload, len))
        }
        Err(e) => Err(e),
    }
}

/// Fetch an HTTPS resource over native TLS 1.3 streaming state machine with progress callback.
pub unsafe fn fetch_https_stream<F>(
    hostname: &str,
    target_path: &str,
    mut on_progress: F,
) -> Result<(&'static [u8], Option<usize>), &'static str>
where
    F: FnMut(usize, Option<usize>),
{
    if !E1000_FOUND {
        return Err("Network card offline");
    }

    let target_ip = crate::dns::resolver::resolve_domain(hostname).unwrap_or([10, 0, 2, 2]);
    let session = tls_connect(hostname)?;

    let mut req_buf = [0u8; 512];
    let mut req_len = 0;
    let req_str = b"GET ";
    req_buf[req_len..req_len + req_str.len()].copy_from_slice(req_str);
    req_len += req_str.len();

    let p_bytes = target_path.as_bytes();
    let to_copy_p = core::cmp::min(p_bytes.len(), 256);
    req_buf[req_len..req_len + to_copy_p].copy_from_slice(&p_bytes[..to_copy_p]);
    req_len += to_copy_p;

    let host_prefix = b" HTTP/1.1\r\nHost: ";
    req_buf[req_len..req_len + host_prefix.len()].copy_from_slice(host_prefix);
    req_len += host_prefix.len();

    let h_bytes = hostname.as_bytes();
    let to_copy_h = core::cmp::min(h_bytes.len(), 128);
    req_buf[req_len..req_len + to_copy_h].copy_from_slice(&h_bytes[..to_copy_h]);
    req_len += to_copy_h;

    let ua_prefix = b"\r\nUser-Agent: ";
    req_buf[req_len..req_len + ua_prefix.len()].copy_from_slice(ua_prefix);
    req_len += ua_prefix.len();

    let ua_bytes = crate::HTTP_USER_AGENT.as_bytes();
    req_buf[req_len..req_len + ua_bytes.len()].copy_from_slice(ua_bytes);
    req_len += ua_bytes.len();

    let req_end = b"\r\nConnection: close\r\n\r\n";
    req_buf[req_len..req_len + req_end.len()].copy_from_slice(req_end);
    req_len += req_end.len();

    let mut enc_buf = [0u8; 512];
    let (enc_len, _tag) = session.encrypt_record(&req_buf[..req_len], &mut enc_buf);

    match crate::tcp::stream::fetch_stream_download(
        target_ip,
        443,
        &enc_buf[..enc_len],
        &mut on_progress,
    ) {
        Ok((payload, cl)) => {
            if payload.len() < 10 || payload[0] == 0x15 {
                Err("TLS 1.3 Alert (Handshake rejected by host)")
            } else {
                Ok((payload, cl))
            }
        }
        Err(e) => Err(e),
    }
}
