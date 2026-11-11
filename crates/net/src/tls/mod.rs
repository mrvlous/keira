// SPDX-License-Identifier: GPL-2.0-only

//! Transport Layer Security (TLS 1.3) cryptographic record and session state machine.

pub mod handshake;
pub mod record;

pub use handshake::{build_hkdf_label, tls_connect};
pub use record::*;

pub mod native {
    pub use super::handshake::{build_hkdf_label, tls_connect};
    pub use super::record::*;
}

#[cfg(test)]
mod tests;
