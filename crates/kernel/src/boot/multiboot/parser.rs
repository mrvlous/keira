// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Multiboot2 information tag parsing implementation.

use super::types::*;
use keira_fs::tar;
use keira_io::vga;

/// Parse Multiboot2 information structure provided by the bootloader.
///
/// # Safety
/// The caller must ensure that `multiboot_info_ptr` points to a valid 8-byte aligned
/// Multiboot2 information structure in mapped kernel address space.
pub unsafe fn parse_multiboot2(multiboot_info_ptr: usize) -> BootPayloadInfo {
    let mut payload = BootPayloadInfo::default();

    if multiboot_info_ptr == 0 {
        return payload;
    }

    let mut addr = multiboot_info_ptr + 8;
    loop {
        let tag_type = *(addr as *const u32);
        let tag_size = *((addr + 4) as *const u32);

        if tag_type == TAG_TYPE_END || tag_size == 0 {
            break;
        }

        match tag_type {
            TAG_TYPE_MODULE => {
                let start = *((addr + 8) as *const u32) as u64;
                let end = *((addr + 12) as *const u32) as u64;
                payload.initrd_start = start;
                payload.initrd_end = end;
                tar::init(start, end);
            }
            TAG_TYPE_BASIC_MEMINFO => {
                payload.mem_lower_kb = *((addr + 8) as *const u32);
                payload.mem_upper_kb = *((addr + 12) as *const u32);
            }
            TAG_TYPE_FRAMEBUFFER => {
                let fb_addr = *((addr + 8) as *const u64);
                let fb_pitch = *((addr + 16) as *const u32);
                let fb_width = *((addr + 20) as *const u32);
                let fb_height = *((addr + 24) as *const u32);
                let fb_bpp = *((addr + 28) as *const u8);

                payload.framebuffer_addr = fb_addr;
                payload.framebuffer_pitch = fb_pitch;
                payload.framebuffer_width = fb_width;
                payload.framebuffer_height = fb_height;
                payload.framebuffer_bpp = fb_bpp;

                vga::FRAMEBUFFER_ADDR = fb_addr;
                vga::FRAMEBUFFER_PITCH = fb_pitch;
                vga::FRAMEBUFFER_WIDTH = fb_width;
                vga::FRAMEBUFFER_HEIGHT = fb_height;
                vga::FRAMEBUFFER_BPP = fb_bpp;
            }
            TAG_TYPE_ACPI_OLD | TAG_TYPE_ACPI_NEW => {
                payload.acpi_rsdp_ptr = (addr + 8) as u64;
            }
            _ => {}
        }

        addr += ((tag_size + 7) & !7) as usize;
    }

    payload
}
