// SPDX-License-Identifier: GPL-2.0-only
/*
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra <mrvlous@proton.me>
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 */

//! TLS record layer definitions and session structures.

pub mod types;

pub use types::{
    TlsSession, TlsState, TLS_AES_128_GCM_SHA256, TLS_CONTENT_APPLICATION_DATA,
    TLS_CONTENT_HANDSHAKE, TLS_VERSION_12, TLS_VERSION_13,
};
