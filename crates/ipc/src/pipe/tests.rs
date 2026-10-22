// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Unit tests for dynamic ring buffer pipes, multi-channel IPC and zero-copy splice.

#[cfg(test)]
mod test {
    use crate::pipe::*;

    #[test]
    fn test_legacy_pipe_read_write_lifecycle() {
        unsafe {
            let (read_fd, write_fd) = create_pipe().expect("create pipe failed");
            assert_eq!(read_fd, 3);
            assert_eq!(write_fd, 4);

            let msg = b"Hello Keira IPC Pipe!";
            let written = write_pipe(msg);
            assert_eq!(written, msg.len());

            let mut out = [0u8; 64];
            let read_bytes = read_pipe(&mut out);
            assert_eq!(read_bytes, msg.len());
            assert_eq!(&out[..read_bytes], msg);

            assert_eq!(read_pipe(&mut out), 0);
        }
    }

    #[test]
    fn test_dynamic_pipe_channel_lifecycle() {
        unsafe {
            pipe_table_reset();
            let pipe_id = create_pipe_channel(0).expect("create_pipe_channel failed");
            assert_eq!(pipe_id, 1);

            let msg = b"Dynamic Pipe Stream Payload";
            let written = write_pipe_channel(pipe_id, msg).expect("write failed");
            assert_eq!(written, msg.len());

            let (active, buffered) = get_pipe_stats();
            assert_eq!(active, 1);
            assert_eq!(buffered, msg.len());

            let mut out = [0u8; 64];
            let read_bytes = read_pipe_channel(pipe_id, &mut out).expect("read failed");
            assert_eq!(read_bytes, msg.len());
            assert_eq!(&out[..read_bytes], msg);

            // Buffer is empty, writer is still open
            assert_eq!(read_pipe_channel(pipe_id, &mut out).unwrap(), 0);

            // Close writer: read should return 0 (POSIX EOF)
            close_pipe_write(pipe_id);
            assert_eq!(read_pipe_channel(pipe_id, &mut out).unwrap(), 0);

            // Close reader: channel is reclaimed
            close_pipe_read(pipe_id);
            let (active_after, _) = get_pipe_stats();
            assert_eq!(active_after, 0);
        }
    }

    #[test]
    fn test_pipe_broken_pipe_epipe() {
        unsafe {
            pipe_table_reset();
            let pipe_id = create_pipe_channel(0).expect("create_pipe_channel failed");

            // Close reader endpoint
            close_pipe_read(pipe_id);

            // Writing to a pipe with no readers returns -EPIPE (-32)
            let result = write_pipe_channel(pipe_id, b"No readers here");
            assert_eq!(result, Err(-32));

            close_pipe_write(pipe_id);
        }
    }

    #[test]
    fn test_pipe_multi_channel_isolation() {
        unsafe {
            pipe_table_reset();
            let p1 = create_pipe_channel(0).expect("p1 create");
            let p2 = create_pipe_channel(0).expect("p2 create");
            assert_ne!(p1, p2);

            write_pipe_channel(p1, b"Alpha").expect("write p1");
            write_pipe_channel(p2, b"Beta").expect("write p2");

            let mut buf1 = [0u8; 16];
            let mut buf2 = [0u8; 16];
            let n1 = read_pipe_channel(p1, &mut buf1).expect("read p1");
            let n2 = read_pipe_channel(p2, &mut buf2).expect("read p2");

            assert_eq!(&buf1[..n1], b"Alpha");
            assert_eq!(&buf2[..n2], b"Beta");

            close_pipe_read(p1);
            close_pipe_write(p1);
            close_pipe_read(p2);
            close_pipe_write(p2);
        }
    }

    #[test]
    fn test_splice_and_vmsplice() {
        assert!(sys_splice(0, 1, 4096, 0).is_ok());
        assert!(sys_splice(0, 1, 0x2000_0000, 0).is_err());

        assert!(sys_vmsplice(0, 0, 4, 0).is_ok());
        assert!(sys_vmsplice(0, 0, 2048, 0).is_err());
    }
}
