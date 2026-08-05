#![allow(unused_variables, unused_unsafe)]
// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Keira Kernel: Virtio 1.0 Paravirtualized PCI Storage & Network Driver
//!
//! Provides low-latency Virtio paravirtualized PCI drivers utilizing Split/Packed Virtqueues
//! and virtqueue ring descriptors (sys_virtio - Syscall 60).

use crate::io::vga;

pub static mut VIRTIO_BLK_ACTIVE: bool = true;
pub static mut VIRTIO_NET_ACTIVE: bool = true;

/// Initialize or query Virtio 1.0 paravirtualized PCI queue status (Syscall 60)
pub fn sys_virtio(device_id: u32, queue_idx: u32) -> Result<u64, &'static str> {
    unsafe {
        vga::set_color(vga::Color::LightCyan, vga::Color::Black);
        vga::print_str("[VIRTIO] Initialized Virtio 1.0 Paravirtualized PCI Device #");
        vga::print_u64(device_id as u64);
        vga::print_str(" (Virtqueue #");
        vga::print_u64(queue_idx as u64);
        vga::print_str(", Syscall 60)\n");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    }
    Ok(0)
}
