// SPDX-License-Identifier: GPL-2.0-only

//! Dynamic Host Configuration Protocol (DHCP) client.

pub mod client;

#[cfg(test)]
mod tests;

pub use client::{dhcp_auto_configure, DhcpConfig, SYSTEM_DHCP};
