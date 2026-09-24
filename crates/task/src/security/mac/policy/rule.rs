// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Mandatory Access Control rule specifications and permission masks.

use super::domain::MacDomain;

pub const MAC_READ: u32 = 0x01;
pub const MAC_WRITE: u32 = 0x02;
pub const MAC_EXEC: u32 = 0x04;
pub const MAC_APPEND: u32 = 0x08;

pub const MAX_MAC_RULES: usize = 16;

/// Mandatory access control policy rule entry.
#[derive(Debug, Copy, Clone)]
pub struct MacRule {
    pub domain: MacDomain,
    pub path_prefix: [u8; 32],
    pub prefix_len: usize,
    pub allowed_mask: u32,
    pub in_use: bool,
}
