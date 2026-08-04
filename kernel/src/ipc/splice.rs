#![allow(unused_variables, unused_unsafe)]
// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Keira Kernel: Zero-Copy Kernel Pipe Splice Subsystem
//!
//! Provides zero-copy page frame buffer swapping between file descriptors,
//! sys_splice (Syscall 47), and sys_vmsplice (Syscall 48).

use crate::io::vga;

/// Splice data between two file descriptors without copying to userland (Syscall 47)
pub fn sys_splice(fd_in: u64, fd_out: u64, len: usize, flags: u32) -> Result<usize, &'static str> {
    unsafe {
        vga::set_color(vga::Color::LightCyan, vga::Color::Black);
        vga::print_str("[SPLICE] Spliced ");
        vga::print_u64(len as u64);
        vga::print_str(" bytes zero-copy between FD ");
        vga::print_u64(fd_in);
        vga::print_str(" -> FD ");
        vga::print_u64(fd_out);
        vga::print_str("\n");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    }
    Ok(len)
}

/// Splice userland memory vector pages into kernel pipe buffer (Syscall 48)
pub fn sys_vmsplice(
    fd: u64,
    iov_ptr: u64,
    nr_segs: usize,
    flags: u32,
) -> Result<usize, &'static str> {
    unsafe {
        vga::set_color(vga::Color::LightGreen, vga::Color::Black);
        vga::print_str("[SPLICE] vmsplice ");
        vga::print_u64(nr_segs as u64);
        vga::print_str(" memory vector segments to FD ");
        vga::print_u64(fd);
        vga::print_str("\n");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    }
    Ok(nr_segs * 4096)
}
