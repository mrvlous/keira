// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Unit tests for POSIX socket lifecycle, non-blocking I/O and buffer operations.

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

#[test]
fn test_loopback_socket_communication() {
    unsafe {
        socket_table_reset();
        crate::driver::loopback::reset_loopback_stats();

        let s1 = create_socket(AF_INET as u64, SOCK_STREAM as u64, 0).expect("Create failed");
        let s2 = create_socket(AF_INET as u64, SOCK_STREAM as u64, 0).expect("Create failed");

        let s1_addr = [0u8, 0, 0x23, 0x82, 127, 0, 0, 1]; // Port 9090, 127.0.0.1
        assert!(connect_socket(s1, s1_addr.as_ptr(), 8).is_ok());

        let s2_addr = [0u8, 0, 0x23, 0x82, 127, 0, 0, 1];
        assert!(connect_socket(s2, s2_addr.as_ptr(), 8).is_ok());

        for slot in SOCKET_TABLE.iter_mut() {
            if slot.in_use && slot.id == s1 as u32 {
                slot.local_port = 9090;
            }
        }

        let msg = b"Keira loopback message!";
        let sent = send_socket(s2, msg.as_ptr(), msg.len()).expect("Send failed");
        assert_eq!(sent, msg.len());

        let (lo_tx, lo_rx, lo_bytes) = crate::driver::loopback::get_loopback_stats();
        assert_eq!(lo_tx, 1);
        assert_eq!(lo_rx, 1);
        assert_eq!(lo_bytes, msg.len() as u64);

        assert!(socket_is_readable(s1));
        let mut rx_buf = [0u8; 64];
        let recvd = recv_socket(s1, rx_buf.as_mut_ptr(), 64).expect("Recv failed");
        assert_eq!(recvd, msg.len());
        assert_eq!(&rx_buf[..recvd], msg);

        assert!(close_socket(s1).is_ok());
        assert!(close_socket(s2).is_ok());
    }
}
