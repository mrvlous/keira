// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! POSIX-compliant socket abstractions and in-kernel socket descriptor table.

#![allow(static_mut_refs)]

use crate::tcp::stream::TcpState;

pub const AF_UNSPEC: u32 = 0;
pub const AF_UNIX: u32 = 1;
pub const AF_INET: u32 = 2;
pub const AF_INET6: u32 = 10;

pub const SOCK_STREAM: u32 = 1;
pub const SOCK_DGRAM: u32 = 2;
pub const SOCK_RAW: u32 = 3;

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

static mut SOCKET_TABLE: [SocketEntry; MAX_SOCKETS] = [SocketEntry::empty(); MAX_SOCKETS];
static mut NEXT_SOCKET_ID: u32 = 1;

/// Create a new network socket descriptor in the kernel socket table.
pub unsafe fn create_socket(
    domain: u64,
    socket_type: u64,
    proto: u64,
) -> Result<u64, &'static str> {
    for slot in SOCKET_TABLE.iter_mut() {
        if !slot.in_use {
            let id = NEXT_SOCKET_ID;
            NEXT_SOCKET_ID = NEXT_SOCKET_ID.wrapping_add(1);
            if NEXT_SOCKET_ID == 0 {
                NEXT_SOCKET_ID = 1;
            }

            *slot = SocketEntry {
                id,
                domain: domain as u32,
                socket_type: socket_type as u32,
                protocol: proto as u32,
                state: TcpState::Closed,
                nonblocking: false,
                local_ip: [10, 0, 2, 15],
                remote_ip: [0; 4],
                local_port: 0,
                remote_port: 0,
                rx_buf: [0; SOCKET_BUF_SIZE],
                rx_len: 0,
                tx_buf: [0; SOCKET_BUF_SIZE],
                tx_len: 0,
                in_use: true,
                is_connected: false,
            };
            return Ok(id as u64);
        }
    }
    Err("Socket table exhausted")
}

/// Connect a socket descriptor to a remote network address.
pub unsafe fn connect_socket(
    sockfd: u64,
    addr_ptr: *const u8,
    len: u64,
) -> Result<(), &'static str> {
    let id = sockfd as u32;
    for slot in SOCKET_TABLE.iter_mut() {
        if slot.in_use && slot.id == id {
            if !addr_ptr.is_null() && len >= 8 {
                let port_bytes = [*(addr_ptr.add(2)), *(addr_ptr.add(3))];
                let port = u16::from_be_bytes(port_bytes);
                let ip = [
                    *(addr_ptr.add(4)),
                    *(addr_ptr.add(5)),
                    *(addr_ptr.add(6)),
                    *(addr_ptr.add(7)),
                ];
                slot.remote_ip = ip;
                slot.remote_port = port;
                slot.local_port = crate::tcp::stream::get_next_src_port();
            }
            slot.state = TcpState::Established;
            slot.is_connected = true;
            return Ok(());
        }
    }
    Err("Invalid socket descriptor")
}

/// Transmit data through an established network socket.
pub unsafe fn send_socket(sockfd: u64, buf: *const u8, len: usize) -> Result<usize, &'static str> {
    if buf.is_null() {
        return Err("Bad buffer pointer");
    }
    let id = sockfd as u32;
    for slot in SOCKET_TABLE.iter_mut() {
        if slot.in_use && slot.id == id {
            if !slot.is_connected {
                return Err("Socket is not connected");
            }
            let avail = SOCKET_BUF_SIZE - slot.tx_len;
            let to_write = core::cmp::min(len, avail);
            if to_write > 0 {
                let dst = slot.tx_buf.as_mut_ptr().add(slot.tx_len);
                core::ptr::copy_nonoverlapping(buf, dst, to_write);
                slot.tx_len += to_write;
            }
            return Ok(to_write);
        }
    }
    Err("Invalid socket descriptor")
}

/// Receive incoming data from a network socket buffer.
pub unsafe fn recv_socket(sockfd: u64, buf: *mut u8, len: usize) -> Result<usize, &'static str> {
    if buf.is_null() {
        return Err("Bad buffer pointer");
    }
    let id = sockfd as u32;
    for slot in SOCKET_TABLE.iter_mut() {
        if slot.in_use && slot.id == id {
            if slot.rx_len == 0 {
                if slot.nonblocking {
                    return Err("Resource temporarily unavailable");
                }
                return Ok(0);
            }
            let to_read = core::cmp::min(len, slot.rx_len);
            core::ptr::copy_nonoverlapping(slot.rx_buf.as_ptr(), buf, to_read);
            if to_read < slot.rx_len {
                let remaining = slot.rx_len - to_read;
                core::ptr::copy(
                    slot.rx_buf.as_ptr().add(to_read),
                    slot.rx_buf.as_mut_ptr(),
                    remaining,
                );
            }
            slot.rx_len -= to_read;
            return Ok(to_read);
        }
    }
    Err("Invalid socket descriptor")
}

/// Close and release a socket descriptor from the active table.
pub unsafe fn close_socket(sockfd: u64) -> Result<(), &'static str> {
    let id = sockfd as u32;
    for slot in SOCKET_TABLE.iter_mut() {
        if slot.in_use && slot.id == id {
            *slot = SocketEntry::empty();
            return Ok(());
        }
    }
    Err("Invalid socket descriptor")
}

/// Check whether the specified socket descriptor has incoming data available or has connected.
pub unsafe fn socket_is_readable(sockfd: u64) -> bool {
    let id = sockfd as u32;
    for slot in SOCKET_TABLE.iter() {
        if slot.in_use && slot.id == id {
            return slot.rx_len > 0 || slot.is_connected;
        }
    }
    false
}

/// Check whether the specified socket descriptor can accept outgoing transmission bytes.
pub unsafe fn socket_is_writable(sockfd: u64) -> bool {
    let id = sockfd as u32;
    for slot in SOCKET_TABLE.iter() {
        if slot.in_use && slot.id == id {
            return slot.is_connected && slot.tx_len < SOCKET_BUF_SIZE;
        }
    }
    false
}

/// Configure non-blocking I/O mode for the specified socket descriptor.
pub unsafe fn set_socket_nonblocking(sockfd: u64, nonblocking: bool) -> Result<(), &'static str> {
    let id = sockfd as u32;
    for slot in SOCKET_TABLE.iter_mut() {
        if slot.in_use && slot.id == id {
            slot.nonblocking = nonblocking;
            return Ok(());
        }
    }
    Err("Invalid socket descriptor")
}

/// Query whether the specified socket descriptor is configured in non-blocking mode.
pub unsafe fn is_socket_nonblocking(sockfd: u64) -> bool {
    let id = sockfd as u32;
    for slot in SOCKET_TABLE.iter() {
        if slot.in_use && slot.id == id {
            return slot.nonblocking;
        }
    }
    false
}

/// Enqueue simulated or driver-received network payload into a socket receive buffer.
pub unsafe fn queue_rx_data(sockfd: u64, data: &[u8]) -> Result<usize, &'static str> {
    let id = sockfd as u32;
    for slot in SOCKET_TABLE.iter_mut() {
        if slot.in_use && slot.id == id {
            let avail = SOCKET_BUF_SIZE - slot.rx_len;
            let to_copy = core::cmp::min(data.len(), avail);
            if to_copy > 0 {
                let dst = slot.rx_buf.as_mut_ptr().add(slot.rx_len);
                core::ptr::copy_nonoverlapping(data.as_ptr(), dst, to_copy);
                slot.rx_len += to_copy;
            }
            return Ok(to_copy);
        }
    }
    Err("Invalid socket descriptor")
}

/// Reset socket table state (used for testing and teardown).
pub unsafe fn socket_table_reset() {
    SOCKET_TABLE = [SocketEntry::empty(); MAX_SOCKETS];
    NEXT_SOCKET_ID = 1;
}

/// Validate TCP socket port bounds.
pub fn validate_socket_port(port: u16) -> bool {
    port != 0
}
