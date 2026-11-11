// SPDX-License-Identifier: GPL-2.0-only

//! TLS record layer definitions and session structures.

pub mod types;

pub use types::{
    TlsSession, TlsState, TLS_AES_128_GCM_SHA256, TLS_CONTENT_APPLICATION_DATA,
    TLS_CONTENT_HANDSHAKE, TLS_VERSION_12, TLS_VERSION_13,
};
