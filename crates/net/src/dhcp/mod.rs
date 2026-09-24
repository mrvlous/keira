// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Dynamic Host Configuration Protocol (DHCP) client.

pub mod client;

#[cfg(test)]
mod tests;

pub use client::{dhcp_auto_configure, DhcpConfig, SYSTEM_DHCP};
