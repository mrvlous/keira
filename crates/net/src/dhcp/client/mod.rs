// SPDX-License-Identifier: GPL-2.0-only

//! DHCP client protocol and configuration models.

pub mod config;
pub mod protocol;

pub use config::{DhcpConfig, SYSTEM_DHCP};
pub use protocol::dhcp_auto_configure;
