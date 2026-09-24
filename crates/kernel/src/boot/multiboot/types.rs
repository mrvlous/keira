// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Multiboot2 tag definitions, magic constants, and boot payload descriptors.

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
