// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! In-kernel socket table manager and handle lifecycle methods.

pub mod table;

pub use table::{
    close_socket, connect_socket, create_socket, is_socket_nonblocking, queue_rx_data, recv_socket,
    send_socket, set_socket_nonblocking, socket_is_readable, socket_is_writable,
    socket_table_reset, NEXT_SOCKET_ID, SOCKET_TABLE,
};
