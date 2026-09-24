// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Unit tests for ring buffer pipes and zero-copy splice.

#[cfg(test)]
mod test {
    use crate::pipe::*;

    #[test]
    fn test_pipe_read_write_lifecycle() {
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
    fn test_splice_and_vmsplice() {
        assert!(sys_splice(0, 1, 4096, 0).is_ok());
        assert!(sys_splice(0, 1, 0x2000_0000, 0).is_err());

        assert!(sys_vmsplice(0, 0, 4, 0).is_ok());
        assert!(sys_vmsplice(0, 0, 2048, 0).is_err());
    }
}
