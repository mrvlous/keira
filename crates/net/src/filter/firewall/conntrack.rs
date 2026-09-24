// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Connection tracking (CONNTRACK) table state and descriptors.

pub const MAX_CONNTRACK_ENTRIES: usize = 16;

/// Stateful connection tracking table entry.
#[derive(Copy, Clone)]
pub struct ConnTrackEntry {
    pub proto: [u8; 8],
    pub src_ip: [u8; 16],
    pub dst_ip: [u8; 16],
    pub dport: u16,
    pub state: [u8; 12],
    pub packets: u32,
    pub in_use: bool,
}
