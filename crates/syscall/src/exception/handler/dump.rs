// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Core dump serialization and filesystem logging for crashed userland tasks.

use crate::exception::handler::signals::{exception_name, signal_name};

pub struct DumpWriter<'a> {
    buf: &'a mut [u8],
    offset: usize,
}

impl<'a> DumpWriter<'a> {
    pub fn new(buf: &'a mut [u8]) -> Self {
        Self { buf, offset: 0 }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.buf[..self.offset]
    }
}

impl<'a> core::fmt::Write for DumpWriter<'a> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let bytes = s.as_bytes();
        let avail = self.buf.len().saturating_sub(self.offset);
        let to_write = bytes.len().min(avail);
        self.buf[self.offset..self.offset + to_write].copy_from_slice(&bytes[..to_write]);
        self.offset += to_write;
        Ok(())
    }
}

/// Generates a structured core dump payload and saves it into the root VFS at `/data/log/core_<PID>.dmp`.
pub fn write_core_dump(
    pid: usize,
    task_name: &str,
    sig: u32,
    vector: u64,
    error_code: u64,
    rip: u64,
    rsp: u64,
    rbp: u64,
    rflags: u64,
    cr2: u64,
) {
    let mut dump_buf = [0u8; 1024];
    let mut writer = DumpWriter::new(&mut dump_buf);
    use core::fmt::Write;
    let _ = core::write!(
        writer,
        "=== KEIRA CORE DUMP ===\n\
         PID: {}\n\
         Name: {}\n\
         Signal: {} ({})\n\
         Vector: {} ({})\n\
         Error Code: 0x{:X}\n\
         RIP: 0x{:X}\n\
         RSP: 0x{:X}\n\
         RBP: 0x{:X}\n\
         RFLAGS: 0x{:X}\n\
         CR2: 0x{:X}\n\
         Status: TERMINATED BY SIGNAL\n",
        pid,
        task_name,
        sig,
        signal_name(sig),
        vector,
        exception_name(vector),
        error_code,
        rip,
        rsp,
        rbp,
        rflags,
        cr2
    );

    let mut path_buf = [0u8; 32];
    let mut p_writer = DumpWriter::new(&mut path_buf);
    let _ = core::write!(p_writer, "/data/log/core_{}.dmp", pid);
    if let Ok(path_str) = core::str::from_utf8(p_writer.as_bytes()) {
        let _ = keira_fs::vfs::create_file(path_str);
        let _ = keira_fs::vfs::write_file(path_str, writer.as_bytes());
    }
}
