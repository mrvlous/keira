// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! HTTP request construction and streaming over TCP layer.

use super::client::{fetch_stream_download, tcp_send_and_receive};
use crate::dns::resolver::resolve_domain;
use crate::driver::e1000::E1000_FOUND;

/// Fetch an HTTP resource using continuous TCP streaming, returns downloaded payload slice and optional Content-Length.
///
/// # Safety
/// Transmits and receives network frames across active NIC interface.
pub unsafe fn fetch_http_stream<F>(
    url: &str,
    on_progress: F,
) -> Result<(&'static [u8], Option<usize>), &'static str>
where
    F: FnMut(usize, Option<usize>),
{
    if !E1000_FOUND {
        return Err("Network card offline");
    }

    let hostname = if let Some(stripped) = url.strip_prefix("http://") {
        stripped
    } else if let Some(stripped) = url.strip_prefix("https://") {
        stripped
    } else {
        url
    };
    let (host, path) = match hostname.find('/') {
        Some(idx) => (&hostname[..idx], &hostname[idx..]),
        None => (hostname, "/"),
    };

    let (host_str, target_port) = if let Some(colon) = host.find(':') {
        let h = &host[..colon];
        let p = host[colon + 1..].parse::<u16>().unwrap_or(80);
        (h, p)
    } else {
        (host, 80)
    };

    let target_ip = resolve_domain(host_str).unwrap_or([10, 0, 2, 2]);

    let mut req_buf = [0u8; 512];
    let mut req_len = 0;
    let req_str = b"GET ";
    req_buf[req_len..req_len + req_str.len()].copy_from_slice(req_str);
    req_len += req_str.len();

    let p_bytes = path.as_bytes();
    let to_copy_p = core::cmp::min(p_bytes.len(), 256);
    req_buf[req_len..req_len + to_copy_p].copy_from_slice(&p_bytes[..to_copy_p]);
    req_len += to_copy_p;

    let host_prefix = b" HTTP/1.1\r\nHost: ";
    req_buf[req_len..req_len + host_prefix.len()].copy_from_slice(host_prefix);
    req_len += host_prefix.len();

    let h_bytes = host.as_bytes();
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

    match fetch_stream_download(target_ip, target_port, &req_buf[..req_len], on_progress) {
        Ok(res) => Ok(res),
        Err(err) => {
            if target_ip != [10, 0, 2, 2] {
                fetch_stream_download(
                    [10, 0, 2, 2],
                    target_port,
                    &req_buf[..req_len],
                    |_cur, _total| {},
                )
            } else {
                Err(err)
            }
        }
    }
}

/// Fetch an HTTP resource over the network stack (Ethernet -> IPv4 -> TCP:80 -> HTTP GET).
///
/// # Safety
/// Transmits and receives network frames across active NIC interface.
pub unsafe fn fetch_http(url: &str) -> Result<([u8; 512], usize), &'static str> {
    if !E1000_FOUND {
        return Err("Network card offline");
    }

    let hostname = if let Some(stripped) = url.strip_prefix("http://") {
        stripped
    } else if let Some(stripped) = url.strip_prefix("https://") {
        stripped
    } else {
        url
    };
    let (host, path) = match hostname.find('/') {
        Some(idx) => (&hostname[..idx], &hostname[idx..]),
        None => (hostname, "/"),
    };

    let (host_str, target_port) = if let Some(colon) = host.find(':') {
        let h = &host[..colon];
        let p = host[colon + 1..].parse::<u16>().unwrap_or(80);
        (h, p)
    } else {
        (host, 80)
    };

    let target_ip = resolve_domain(host_str).unwrap_or([10, 0, 2, 2]);

    let mut req_buf = [0u8; 512];
    let mut req_len = 0;
    let req_str = b"GET ";
    req_buf[req_len..req_len + req_str.len()].copy_from_slice(req_str);
    req_len += req_str.len();

    let p_bytes = path.as_bytes();
    let to_copy_p = core::cmp::min(p_bytes.len(), 128);
    req_buf[req_len..req_len + to_copy_p].copy_from_slice(&p_bytes[..to_copy_p]);
    req_len += to_copy_p;

    let host_prefix = b" HTTP/1.1\r\nHost: ";
    req_buf[req_len..req_len + host_prefix.len()].copy_from_slice(host_prefix);
    req_len += host_prefix.len();

    let h_bytes = host.as_bytes();
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

    match tcp_send_and_receive(target_ip, target_port, &req_buf[..req_len]) {
        Ok(res) => Ok(res),
        Err(err) => {
            if target_ip != [10, 0, 2, 2] {
                tcp_send_and_receive([10, 0, 2, 2], target_port, &req_buf[..req_len])
            } else {
                Err(err)
            }
        }
    }
}
