// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! VFAT Long File Name (LFN) 8.3 checksum calculation.

/// Computes standard 8-bit checksum over an 11-byte short filename directory entry name.
pub fn lfn_checksum(sfn: &[u8; 11]) -> u8 {
    let mut sum: u8 = 0;
    for &b in sfn {
        sum = (((sum & 1) << 7) | (sum >> 1)).wrapping_add(b);
    }
    sum
}
