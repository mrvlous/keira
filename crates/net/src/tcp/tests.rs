// SPDX-License-Identifier: GPL-2.0-only

//! Unit tests for TCP flags and ephemeral port allocator.

use super::*;

#[test]
fn test_tcp_flags_constants() {
    assert_eq!(TCP_FLAG_FIN, 0x01);
    assert_eq!(TCP_FLAG_SYN, 0x02);
    assert_eq!(TCP_FLAG_RST, 0x04);
    assert_eq!(TCP_FLAG_PSH, 0x08);
    assert_eq!(TCP_FLAG_ACK, 0x10);
}

#[test]
fn test_get_next_src_port() {
    let p1 = get_next_src_port();
    let p2 = get_next_src_port();
    assert!(p1 >= 49152);
    assert_ne!(p1, p2);
}
