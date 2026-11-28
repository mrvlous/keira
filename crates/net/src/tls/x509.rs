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

//! Lightweight DER ASN.1 X.509 certificate metadata and SAN DNS validator.

const OID_SUBJECT_ALT_NAME: [u8; 5] = [0x06, 0x03, 0x55, 0x1D, 0x11];
const OID_COMMON_NAME: [u8; 5] = [0x06, 0x03, 0x55, 0x04, 0x03];

/// Read an ASN.1 DER tag length starting at `pos` returning `(len, next_pos)`.
fn read_der_len(data: &[u8], pos: usize) -> Option<(usize, usize)> {
    let b = *data.get(pos)?;
    if (b & 0x80) == 0 {
        return Some((b as usize, pos + 1));
    }
    let num_octets = (b & 0x7F) as usize;
    if num_octets == 0 || num_octets > 4 || pos + 1 + num_octets > data.len() {
        return None;
    }
    let mut len = 0usize;
    for i in 0..num_octets {
        len = (len << 8) | (data[pos + 1 + i] as usize);
    }
    Some((len, pos + 1 + num_octets))
}

/// Match a DNS pattern (supporting single-label wildcard `*.`) against `host`.
pub fn match_dns_pattern(pattern: &str, host: &str) -> bool {
    if pattern.is_empty() || host.is_empty() {
        return false;
    }
    if pattern.eq_ignore_ascii_case(host) {
        return true;
    }
    if let Some(suffix) = pattern.strip_prefix("*.") {
        if let Some(dot_idx) = host.find('.') {
            let tail = &host[dot_idx + 1..];
            return !tail.is_empty() && tail.eq_ignore_ascii_case(suffix);
        }
    }
    false
}

/// Check whether `name` parses as a dotted IPv4 literal.
pub fn is_ipv4_literal(name: &str) -> bool {
    let mut dots = 0usize;
    let mut digits = 0usize;
    for b in name.bytes() {
        if b == b'.' {
            if digits == 0 {
                return false;
            }
            dots += 1;
            digits = 0;
        } else if b.is_ascii_digit() {
            digits += 1;
        } else {
            return false;
        }
    }
    dots == 3 && digits > 0
}

/// Parse TLS 1.3 `Certificate` body, extract primary CN/SAN and verify hostname if required.
pub fn verify_certificate_payload(
    cert_msg_body: &[u8],
    expected_host: &str,
    allow_insecure: bool,
    cn_out: &mut [u8; 128],
    cn_len_out: &mut usize,
) -> Result<(), &'static str> {
    if cert_msg_body.len() < 4 {
        return Err("TLS 1.3 Certificate payload truncated");
    }
    let ctx_len = cert_msg_body[0] as usize;
    let list_start = 1 + ctx_len;
    if list_start + 3 > cert_msg_body.len() {
        return Err("TLS 1.3 Certificate list header truncated");
    }
    let list_len = ((cert_msg_body[list_start] as usize) << 16)
        | ((cert_msg_body[list_start + 1] as usize) << 8)
        | (cert_msg_body[list_start + 2] as usize);
    let entries_start = list_start + 3;
    if list_len < 4 || entries_start + list_len > cert_msg_body.len() {
        return Err("TLS 1.3 empty or malformed certificate chain");
    }

    let entry_len = ((cert_msg_body[entries_start] as usize) << 16)
        | ((cert_msg_body[entries_start + 1] as usize) << 8)
        | (cert_msg_body[entries_start + 2] as usize);
    let cert_start = entries_start + 3;
    if entry_len < 16 || cert_start + entry_len > cert_msg_body.len() {
        return Err("TLS 1.3 malformed leaf X.509 certificate");
    }
    let leaf_der = &cert_msg_body[cert_start..cert_start + entry_len];
    if leaf_der[0] != 0x30 {
        return Err("TLS 1.3 invalid X.509 DER sequence");
    }

    *cn_len_out = 0;
    let mut san_matched = false;
    let mut first_san_saved = false;

    // Scan DER byte slice for SubjectAltName OID (2.5.29.17) and CommonName OID (2.5.4.3)
    let mut idx = 0usize;
    while idx + OID_SUBJECT_ALT_NAME.len() < leaf_der.len() {
        if leaf_der[idx..idx + OID_SUBJECT_ALT_NAME.len()] == OID_SUBJECT_ALT_NAME {
            let mut p = idx + OID_SUBJECT_ALT_NAME.len();
            // Skip optional critical boolean (0x01 0x01 0xFF)
            if leaf_der.get(p) == Some(&0x01) && leaf_der.get(p + 1) == Some(&0x01) {
                p += 3;
            }
            if leaf_der.get(p) == Some(&0x04) {
                if let Some((oct_len, seq_pos)) = read_der_len(leaf_der, p + 1) {
                    let oct_end = (seq_pos + oct_len).min(leaf_der.len());
                    if leaf_der.get(seq_pos) == Some(&0x30) {
                        if let Some((seq_len, mut gn_pos)) = read_der_len(leaf_der, seq_pos + 1) {
                            let seq_end = (gn_pos + seq_len).min(oct_end);
                            while gn_pos < seq_end {
                                let tag = leaf_der[gn_pos];
                                if let Some((gn_len, val_pos)) = read_der_len(leaf_der, gn_pos + 1)
                                {
                                    let val_end = (val_pos + gn_len).min(seq_end);
                                    if tag == 0x82 {
                                        if let Ok(dns_name) =
                                            core::str::from_utf8(&leaf_der[val_pos..val_end])
                                        {
                                            if !first_san_saved {
                                                let copy_len = dns_name.len().min(cn_out.len());
                                                cn_out[..copy_len].copy_from_slice(
                                                    &dns_name.as_bytes()[..copy_len],
                                                );
                                                *cn_len_out = copy_len;
                                                first_san_saved = true;
                                            }
                                            if match_dns_pattern(dns_name, expected_host) {
                                                san_matched = true;
                                            }
                                        }
                                    }
                                    gn_pos = val_end;
                                } else {
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        } else if leaf_der[idx..idx + OID_COMMON_NAME.len()] == OID_COMMON_NAME {
            let val_tag_pos = idx + OID_COMMON_NAME.len();
            if val_tag_pos + 2 <= leaf_der.len() {
                if let Some((cn_len, val_pos)) = read_der_len(leaf_der, val_tag_pos + 1) {
                    let val_end = (val_pos + cn_len).min(leaf_der.len());
                    if let Ok(cn_str) = core::str::from_utf8(&leaf_der[val_pos..val_end]) {
                        if *cn_len_out == 0 {
                            let copy_len = cn_str.len().min(cn_out.len());
                            cn_out[..copy_len].copy_from_slice(&cn_str.as_bytes()[..copy_len]);
                            *cn_len_out = copy_len;
                        }
                        if match_dns_pattern(cn_str, expected_host) {
                            san_matched = true;
                        }
                    }
                }
            }
        }
        idx += 1;
    }

    if !allow_insecure
        && !expected_host.is_empty()
        && !is_ipv4_literal(expected_host)
        && !san_matched
    {
        return Err("TLS 1.3 certificate SAN/CN does not match target host");
    }

    Ok(())
}
