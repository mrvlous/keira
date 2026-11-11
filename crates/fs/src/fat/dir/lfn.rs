// SPDX-License-Identifier: GPL-2.0-only

//! VFAT Long File Name (LFN) 8.3 checksum calculation.

/// Computes standard 8-bit checksum over an 11-byte short filename directory entry name.
pub fn lfn_checksum(sfn: &[u8; 11]) -> u8 {
    let mut sum: u8 = 0;
    for &b in sfn {
        sum = (((sum & 1) << 7) | (sum >> 1)).wrapping_add(b);
    }
    sum
}
