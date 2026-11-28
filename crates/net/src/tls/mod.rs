// SPDX-License-Identifier: GPL-2.0-only
//
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

//! Transport Layer Security (TLS 1.3) cryptographic record, handshake and session state machine.

pub mod handshake;
pub mod keyschedule;
pub mod record;
pub mod x509;

pub use handshake::{
    build_hkdf_label, tls_attach_tcp, tls_close_slot, tls_connect, tls_recv_slot, tls_send_slot,
};
pub use record::*;

pub mod native {
    pub use super::handshake::{
        build_hkdf_label, tls_attach_tcp, tls_close_slot, tls_connect, tls_recv_slot, tls_send_slot,
    };
    pub use super::record::*;
}

#[cfg(test)]
mod tests;
