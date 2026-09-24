// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Environment variable interpolation and expansion routines.

use crate::state::env::get_env_var;

/// Expands `$VAR` references in `input` writing output into `out_buf` and returns length.
pub fn expand_env_vars(input: &str, out_buf: &mut [u8]) -> usize {
    let mut out_idx = 0usize;
    let bytes = input.as_bytes();
    let len = bytes.len();
    let mut i = 0usize;

    while i < len {
        if bytes[i] == b'$' && i + 1 < len {
            let start = i + 1;
            let mut end = start;
            while end < len && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'_') {
                end += 1;
            }
            if end > start {
                if let Ok(var_name) = core::str::from_utf8(&bytes[start..end]) {
                    let mut val_buf = [0u8; 128];
                    if let Ok(vlen) = unsafe { get_env_var(var_name, &mut val_buf) } {
                        let to_copy = core::cmp::min(vlen, out_buf.len() - out_idx);
                        out_buf[out_idx..out_idx + to_copy].copy_from_slice(&val_buf[..to_copy]);
                        out_idx += to_copy;
                        i = end;
                        continue;
                    }
                }
            }
        }
        if out_idx < out_buf.len() {
            out_buf[out_idx] = bytes[i];
            out_idx += 1;
        }
        i += 1;
    }
    out_idx
}
