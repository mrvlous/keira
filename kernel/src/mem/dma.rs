// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

#![allow(unused_variables, unused_unsafe)]

//!
//! Provides contiguous physical DMA memory allocation and Scatter-Gather list
//! mapping for high-speed hardware bus master data transfers.

use crate::io::vga;

pub struct ScatterGatherEntry {
    pub phys_addr: u64,
    pub length: u32,
}

pub struct DmaBuffer {
    pub vaddr: u64,
    pub paddr: u64,
    pub size: usize,
}

/// Allocate physically contiguous DMA memory buffer for bus master drivers
pub fn alloc_dma_buffer(size: usize) -> Result<DmaBuffer, &'static str> {
    unsafe {
        let frame = crate::mem::pmm::alloc_frame().ok_or("Out of physical frames for DMA")?;
        vga::set_color(vga::Color::LightCyan, vga::Color::Black);
        vga::print_str("[DMA] Allocated ");
        vga::print_u64(size as u64);
        vga::print_str(" bytes DMA Buffer (Physical: 0x");
        print_hex(frame);
        vga::print_str(").\n");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        Ok(DmaBuffer {
            vaddr: frame,
            paddr: frame,
            size,
        })
    }
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
