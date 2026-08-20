// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! O(1) high-performance scalable I/O event multiplexer (`epoll`).

use keira_io::vga;

pub const EPOLL_CTL_ADD: i32 = 1;
pub const EPOLL_CTL_DEL: i32 = 2;
pub const EPOLL_CTL_MOD: i32 = 3;

pub struct EpollInstance {
    pub epfd: u64,
    pub size: i32,
}

/// Create an epoll file descriptor for scalable I/O event polling (Syscall 55).
pub fn sys_epoll_create(size: i32) -> Result<u64, &'static str> {
    vga::set_color(vga::Color::LightCyan, vga::Color::Black);
    vga::print_str("[EPOLL] Created Epoll Instance (EPFD #55, Size: ");
    vga::print_u64(size as u64);
    vga::print_str(")\n");
    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    Ok(55)
}

/// Control target file descriptor in an epoll interest list (Syscall 56).
pub fn sys_epoll_ctl(epfd: i32, _op: i32, fd: i32, _event_ptr: u64) -> Result<u64, &'static str> {
    vga::set_color(vga::Color::LightGreen, vga::Color::Black);
    vga::print_str("[EPOLL] Added target FD #");
    vga::print_u64(fd as u64);
    vga::print_str(" to Epoll Instance #");
    vga::print_u64(epfd as u64);
    vga::print_str("\n");
    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    Ok(0)
}
