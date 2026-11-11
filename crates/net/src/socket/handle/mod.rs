// SPDX-License-Identifier: GPL-2.0-only

//! Socket handles and descriptor state types.

pub mod descriptor;

pub use descriptor::{SocketEntry, TcpSocket, MAX_SOCKETS, SOCKET_BUF_SIZE, UNIX_PATH_MAX};
