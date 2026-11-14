// SPDX-License-Identifier: GPL-2.0-only
/*
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra <mrvlous@proton.me>
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 */

//! Multiboot2 tag definitions, magic constants and boot payload descriptors.

pub const MULTIBOOT2_BOOTLOADER_MAGIC: u32 = 0x36d76289;

pub const TAG_TYPE_END: u32 = 0;
pub const TAG_TYPE_CMDLINE: u32 = 1;
pub const TAG_TYPE_BOOT_LOADER_NAME: u32 = 2;
pub const TAG_TYPE_MODULE: u32 = 3;
pub const TAG_TYPE_BASIC_MEMINFO: u32 = 4;
pub const TAG_TYPE_BOOTDEV: u32 = 5;
pub const TAG_TYPE_MMAP: u32 = 6;
pub const TAG_TYPE_VBE: u32 = 7;
pub const TAG_TYPE_FRAMEBUFFER: u32 = 8;
pub const TAG_TYPE_ELF_SECTIONS: u32 = 9;
pub const TAG_TYPE_APM: u32 = 10;
pub const TAG_TYPE_EFI32: u32 = 11;
pub const TAG_TYPE_EFI64: u32 = 12;
pub const TAG_TYPE_SMBIOS: u32 = 13;
pub const TAG_TYPE_ACPI_OLD: u32 = 14;
pub const TAG_TYPE_ACPI_NEW: u32 = 15;
pub const TAG_TYPE_NETWORK: u32 = 16;
pub const TAG_TYPE_LOAD_BASE_ADDR: u32 = 21;

/// Parsed kernel boot payload information extracted from Multiboot2 tags.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct BootPayloadInfo {
    pub initrd_start: u64,
    pub initrd_end: u64,
    pub acpi_rsdp_ptr: u64,
    pub mem_lower_kb: u32,
    pub mem_upper_kb: u32,
    pub framebuffer_addr: u64,
    pub framebuffer_pitch: u32,
    pub framebuffer_width: u32,
    pub framebuffer_height: u32,
    pub framebuffer_bpp: u8,
}

use keira_fs::tar;
use keira_io::vga;

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

pub mod types {
    pub use super::*;
}

pub mod parser {
    pub use super::parse_multiboot2;
}
