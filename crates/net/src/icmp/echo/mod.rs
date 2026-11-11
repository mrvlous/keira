// SPDX-License-Identifier: GPL-2.0-only

//! ICMP echo request handling.

pub mod ping;

pub use ping::send_ping;
