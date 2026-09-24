// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! VFAT Long File Name (LFN) on-disk entry structures and multi-part accumulator.

/// 32-byte VFAT Long File Name (LFN) directory record.
#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
pub struct LfnEntry {
    /// Sequence number of this LFN component (0x40 bit set for final physical entry).
    pub sequence: u8,
    /// Characters 1 through 5 of the long-name sub-component (UTF-16LE).
    pub name_part1: [u16; 5],
    /// File attributes (always 0x0F: Read-only, Hidden, System, Volume ID).
    pub attr: u8,
    /// Type of LFN record (always 0 for name components).
    pub lfn_type: u8,
    /// Checksum of the associated 8.3 short directory entry name.
    pub checksum: u8,
    /// Characters 6 through 11 of the long-name sub-component (UTF-16LE).
    pub name_part2: [u16; 6],
    /// Meaningless in LFN; must be zero.
    pub first_cluster: u16,
    /// Characters 12 through 13 of the long-name sub-component (UTF-16LE).
    pub name_part3: [u16; 2],
}

/// Helper accumulator for decoding multipart UTF-16 Long File Names.
pub struct LfnAccumulator {
    /// Buffer holding up to 260 UTF-16 code units.
    pub chars: [u16; 260],
    /// Indicates whether an LFN sequence is currently being actively parsed.
    pub active: bool,
    /// Highest LFN sequence index encountered in the current chain.
    pub max_index: usize,
}

impl Default for LfnAccumulator {
    fn default() -> Self {
        Self::new()
    }
}

impl LfnAccumulator {
    /// Constructs a clean, inactive LFN accumulator instance.
    pub const fn new() -> Self {
        Self {
            chars: [0u16; 260],
            active: false,
            max_index: 0,
        }
    }

    /// Resets the accumulator state for the next directory record sequence.
    pub fn reset(&mut self) {
        self.chars = [0u16; 260];
        self.active = false;
        self.max_index = 0;
    }
}
