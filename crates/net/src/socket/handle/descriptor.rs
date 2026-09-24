// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Socket descriptor representations and state blocks.

use crate::tcp::state::TcpState;

pub const MAX_SOCKETS: usize = 16;
pub const SOCKET_BUF_SIZE: usize = 2048;

/// Legacy TCP socket structure maintained for backward compatibility.
pub struct TcpSocket {
    pub state: TcpState,
    pub local_ip: [u8; 4],
    pub remote_ip: [u8; 4],
    pub local_port: u16,
    pub remote_port: u16,
    pub seq_num: u32,
    pub ack_num: u32,
}

impl TcpSocket {
    /// Construct a new legacy TCP socket.
    pub fn new(local_ip: [u8; 4], remote_ip: [u8; 4], local_port: u16, remote_port: u16) -> Self {
        Self {
            state: TcpState::Closed,
            local_ip,
            remote_ip,
            local_port,
            remote_port,
            seq_num: 1000,
            ack_num: 0,
        }
    }
}

/// In-kernel network socket descriptor entry.
#[derive(Clone, Copy, Debug)]
pub struct SocketEntry {
    pub id: u32,
    pub domain: u32,
    pub socket_type: u32,
    pub protocol: u32,
    pub state: TcpState,
    pub nonblocking: bool,
    pub local_ip: [u8; 4],
    pub remote_ip: [u8; 4],
    pub local_port: u16,
    pub remote_port: u16,
    pub rx_buf: [u8; SOCKET_BUF_SIZE],
    pub rx_len: usize,
    pub tx_buf: [u8; SOCKET_BUF_SIZE],
    pub tx_len: usize,
    pub in_use: bool,
    pub is_connected: bool,
}

impl SocketEntry {
    /// Construct an empty, unused socket entry slot.
    pub const fn empty() -> Self {
        Self {
            id: 0,
            domain: 0,
            socket_type: 0,
            protocol: 0,
            state: TcpState::Closed,
            nonblocking: false,
            local_ip: [0; 4],
            remote_ip: [0; 4],
            local_port: 0,
            remote_port: 0,
            rx_buf: [0; SOCKET_BUF_SIZE],
            rx_len: 0,
            tx_buf: [0; SOCKET_BUF_SIZE],
            tx_len: 0,
            in_use: false,
            is_connected: false,
        }
    }
}
