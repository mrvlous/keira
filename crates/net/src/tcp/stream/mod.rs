// SPDX-License-Identifier: GPL-2.0-only

//! Continuous TCP stream connection and packet transmission.

pub mod client;

pub use client::{parse_tcp_payload, tcp_send_and_receive};
