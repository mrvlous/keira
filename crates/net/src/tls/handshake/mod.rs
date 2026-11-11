// SPDX-License-Identifier: GPL-2.0-only

//! TLS 1.3 handshake negotiation and cryptographic key exchange.

pub mod client;
pub mod session;

pub use client::{build_hkdf_label, tls_connect};
