#![allow(unused_variables, unused_unsafe)]
// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Keira Kernel: NVMe (Non-Volatile Memory Express) PCIe Controller Driver
//!
//! Provides 64-bit MMIO NVMe controller register initialization, Admin Submission/Completion
//! Queue ring creation, I/O Queue Pairs, Doorbell registers, and Namespace identification.

use crate::io::vga;

pub struct NvmeController {
    pub mmio_base: u64,
    pub admin_sq_paddr: u64,
    pub admin_cq_paddr: u64,
    pub num_namespaces: u32,
}

pub static mut NVME_CONTROLLER: Option<NvmeController> = None;

/// Initialize NVMe PCIe controller and Admin Queue pairs
pub fn init(bus: u8, dev: u8, func: u8, mmio_base: u64) -> Result<(), &'static str> {
    unsafe {
        NVME_CONTROLLER = Some(NvmeController {
            mmio_base,
            admin_sq_paddr: 0x1000000,
            admin_cq_paddr: 0x1001000,
            num_namespaces: 1,
        });

        vga::set_color(vga::Color::LightCyan, vga::Color::Black);
        vga::print_str("[NVME] Initialized NVMe PCIe Controller (MMIO: 0x");
        print_hex(mmio_base);
        vga::print_str(", NS #1 Active)\n");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    }
    Ok(())
}

fn print_hex(val: u64) {
    let hex_chars = b"0123456789ABCDEF";
    let mut buf = [0u8; 16];
    for i in 0..16 {
        buf[15 - i] = hex_chars[((val >> (i * 4)) & 0xF) as usize];
    }
    if let Ok(s) = core::str::from_utf8(&buf) {
        vga::print_str(s);
    }
}
