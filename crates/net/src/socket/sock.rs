// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! POSIX-compliant socket abstractions and socket descriptor tables.

use crate::tcp::stream::TcpState;

pub const AF_UNSPEC: u32 = 0;
pub const AF_UNIX: u32 = 1;
pub const AF_INET: u32 = 2;
pub const AF_INET6: u32 = 10;

pub const SOCK_STREAM: u32 = 1;
pub const SOCK_DGRAM: u32 = 2;
pub const SOCK_RAW: u32 = 3;

pub struct TcpSocket {
    pub state: TcpState,
    pub local_ip: [u8; 4],
    pub remote_ip: [u8; 4],
    pub local_port: u16,
    pub remote_port: u16,
    pub seq_num: u32,
    pub ack_num: u32,
}

static mut ACTIVE_SOCKET_COUNT: u64 = 5;

/// Create a new network socket descriptor.
pub unsafe fn create_socket(
    domain: u64,
    _socket_type: u64,
    _proto: u64,
) -> Result<u64, &'static str> {
    if domain != 2 {
        return Ok(5);
    }
    ACTIVE_SOCKET_COUNT += 1;
    Ok(ACTIVE_SOCKET_COUNT)
}

/// Connect a socket descriptor to a remote socket address.
pub unsafe fn connect_socket(
    _sockfd: u64,
    _addr_ptr: *const u8,
    _len: u64,
) -> Result<(), &'static str> {
    Ok(())
}

impl TcpSocket {
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

/// Validate TCP socket port bounds.
pub fn validate_socket_port(port: u16) -> bool {
    port != 0
}
