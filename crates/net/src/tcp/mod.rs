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

//! Transmission Control Protocol (TCP) stream transport and state management.

pub mod header;
pub mod state;
pub mod stream;

#[cfg(test)]
mod tests;

pub use header::{
    tcp_checksum, TcpHeader, TCP_FLAG_ACK, TCP_FLAG_FIN, TCP_FLAG_PSH, TCP_FLAG_RST, TCP_FLAG_SYN,
};
pub use state::{get_next_src_port, TcpState};
pub use stream::{parse_tcp_payload, tcp_send_and_receive};
