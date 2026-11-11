// SPDX-License-Identifier: GPL-2.0-only

//! POSIX socket address families and socket type enumerations.

pub const AF_UNSPEC: u32 = 0;
pub const AF_UNIX: u32 = 1;
pub const AF_INET: u32 = 2;
pub const AF_INET6: u32 = 10;

pub const SOCK_STREAM: u32 = 1;
pub const SOCK_DGRAM: u32 = 2;
pub const SOCK_RAW: u32 = 3;

/// Validate TCP socket port bounds.
pub fn validate_socket_port(port: u16) -> bool {
    port != 0
}
