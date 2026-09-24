// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Continuous TCP stream connection, HTTP operations, and chunked decoding.

pub mod client;
pub mod http;

pub use client::{
    dechunk_in_place, fetch_stream_download, parse_tcp_payload, tcp_send_and_receive,
    STREAM_BUFFER_CAPACITY, STREAM_DOWNLOAD_BUFFER,
};
pub use http::{fetch_http, fetch_http_stream};
