// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! POSIX socket abstractions and kernel table management.

pub mod family;
pub mod handle;
pub mod manager;

#[cfg(test)]
mod tests;

pub use family::{
    validate_socket_port, AF_INET, AF_INET6, AF_UNIX, AF_UNSPEC, SOCK_DGRAM, SOCK_RAW, SOCK_STREAM,
};
pub use handle::{SocketEntry, TcpSocket, MAX_SOCKETS, SOCKET_BUF_SIZE};
pub use manager::{
    close_socket, connect_socket, create_socket, is_socket_nonblocking, queue_rx_data, recv_socket,
    send_socket, set_socket_nonblocking, socket_is_readable, socket_is_writable,
    socket_table_reset, NEXT_SOCKET_ID, SOCKET_TABLE,
};

pub mod sock {
    pub use super::family::{
        validate_socket_port, AF_INET, AF_INET6, AF_UNIX, AF_UNSPEC, SOCK_DGRAM, SOCK_RAW,
        SOCK_STREAM,
    };
    pub use super::handle::{SocketEntry, TcpSocket, MAX_SOCKETS, SOCKET_BUF_SIZE};
    pub use super::manager::{
        close_socket, connect_socket, create_socket, is_socket_nonblocking, queue_rx_data,
        recv_socket, send_socket, set_socket_nonblocking, socket_is_readable, socket_is_writable,
        socket_table_reset, NEXT_SOCKET_ID, SOCKET_TABLE,
    };
}
