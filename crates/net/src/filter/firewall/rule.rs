// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Netfilter firewall rule structures and command vector constants.

pub const NETFILTER_CMD_STATUS: u32 = 1;
pub const NETFILTER_CMD_ADD_RULE: u32 = 2;
pub const NETFILTER_CMD_DEL_RULE: u32 = 3;
pub const NETFILTER_CMD_FLUSH: u32 = 4;
pub const NETFILTER_CMD_TOGGLE: u32 = 5;

pub const MAX_FIREWALL_RULES: usize = 16;

/// Netfilter stateful firewall packet filtering rule.
#[derive(Copy, Clone)]
pub struct FirewallRule {
    pub chain: [u8; 12],
    pub proto: [u8; 8],
    pub src_ip: [u8; 16],
    pub dst_ip: [u8; 16],
    pub dport: u16,
    pub action: [u8; 12],
    pub match_count: u32,
    pub in_use: bool,
}
