// SPDX-License-Identifier: GPL-2.0-only

//! POSIX socket domains and type specifications.

pub mod domains;

pub use domains::{
    validate_socket_port, AF_INET, AF_INET6, AF_UNIX, AF_UNSPEC, SOCK_DGRAM, SOCK_RAW, SOCK_STREAM,
};
