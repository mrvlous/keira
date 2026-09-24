// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Unit tests for POSIX socket lifecycle, non-blocking I/O, and buffer operations.

use super::*;

#[test]
fn test_socket_creation_and_lifecycle() {
    unsafe {
        socket_table_reset();

        let sockfd = create_socket(AF_INET as u64, SOCK_STREAM as u64, 0).expect("Create failed");
        assert!(sockfd > 0);

        let dummy_sockaddr = [0u8, 0, 0x1f, 0x90, 10, 0, 2, 2]; // Port 8080, 10.0.2.2
        assert!(connect_socket(sockfd, dummy_sockaddr.as_ptr(), 8).is_ok());

        assert!(socket_is_writable(sockfd));

        let data = b"ping";
        assert_eq!(queue_rx_data(sockfd, data), Ok(4));
        assert!(socket_is_readable(sockfd));

        let mut buf = [0u8; 16];
        let bytes = recv_socket(sockfd, buf.as_mut_ptr(), 16).expect("Recv failed");
        assert_eq!(bytes, 4);
        assert_eq!(&buf[..4], b"ping");

        assert!(close_socket(sockfd).is_ok());
    }
}
