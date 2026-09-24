// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Transmission Control Protocol (TCP) stream transport and HTTP client.

pub mod header;
pub mod state;
pub mod stream;

#[cfg(test)]
mod tests;

pub use header::{
    tcp_checksum, TcpHeader, TCP_FLAG_ACK, TCP_FLAG_FIN, TCP_FLAG_PSH, TCP_FLAG_RST, TCP_FLAG_SYN,
};
pub use state::{get_next_src_port, TcpState};
pub use stream::{
    dechunk_in_place, fetch_http, fetch_http_stream, fetch_stream_download, parse_tcp_payload,
    tcp_send_and_receive, STREAM_BUFFER_CAPACITY, STREAM_DOWNLOAD_BUFFER,
};
