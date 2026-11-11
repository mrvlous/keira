// SPDX-License-Identifier: GPL-2.0-only

//! In-kernel socket table manager and handle lifecycle methods.

pub mod table;

pub use table::{
    bind_socket, close_socket, connect_socket, create_socket, create_socketpair,
    for_each_unix_socket, format_proc_raw, format_proc_sockstat, format_proc_tcp, format_proc_udp,
    format_proc_unix, is_socket_nonblocking, queue_rx_data, recv_socket, send_socket,
    set_socket_nonblocking, socket_is_readable, socket_is_writable, socket_table_reset,
    NEXT_SOCKET_ID, SOCKET_TABLE,
};
