// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Service configuration parser and serializer for .conf files.

use crate::service::daemon::types::ServiceRecord;

/// Parse an unsigned 32-bit integer from a string slice without stdlib.
pub fn parse_u32(s: &str) -> Option<u32> {
    let mut val: u32 = 0;
    if s.is_empty() {
        return None;
    }
    for b in s.bytes() {
        if !b.is_ascii_digit() {
            return None;
        }
        val = val.checked_mul(10)?.checked_add((b - b'0') as u32)?;
    }
    Some(val)
}

/// Parse a .conf key-value buffer and apply settings to a service record.
pub fn parse_conf_into_record(content: &str, record: &mut ServiceRecord) {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with(';') {
            continue;
        }
        if let Some(eq_idx) = trimmed.find('=') {
            let key = trimmed[..eq_idx].trim();
            let val = trimmed[eq_idx + 1..].trim();

            match key {
                "name" => {
                    let klen = val.len().min(16);
                    record.name[..klen].copy_from_slice(&val.as_bytes()[..klen]);
                    record.name_len = klen;
                }
                "description" => {
                    let dlen = val.len().min(48);
                    record.desc[..dlen].copy_from_slice(&val.as_bytes()[..dlen]);
                    record.desc_len = dlen;
                }
                "enabled" => {
                    record.enabled = val == "1" || val == "true" || val == "yes";
                }
                "auto_restart" => {
                    record.auto_restart = val == "1" || val == "true" || val == "yes";
                }
                "port" => {
                    if let Some(p) = parse_u32(val) {
                        record.port = p as u16;
                    }
                }
                "interval" | "interval_secs" => {
                    if let Some(iv) = parse_u32(val) {
                        record.interval_secs = iv;
                    }
                }
                _ => {}
            }
        }
    }
}
