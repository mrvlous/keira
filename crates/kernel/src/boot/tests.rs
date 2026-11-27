// SPDX-License-Identifier: GPL-2.0-only
//
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

//! Unit tests for kernel boot and multiboot2 structures.

use super::*;

#[test]
fn test_boot_payload_default() {
    let payload = BootPayloadInfo::default();
    assert_eq!(payload.initrd_start, 0);
    assert_eq!(payload.initrd_end, 0);
    assert_eq!(payload.acpi_rsdp_ptr, 0);
    assert_eq!(payload.framebuffer_addr, 0);
}

#[test]
fn test_multiboot2_constants() {
    assert_eq!(MULTIBOOT2_BOOTLOADER_MAGIC, 0x36d76289);
    assert_eq!(TAG_TYPE_END, 0);
    assert_eq!(TAG_TYPE_CMDLINE, 1);
    assert_eq!(TAG_TYPE_MODULE, 3);
    assert_eq!(TAG_TYPE_FRAMEBUFFER, 8);
}

#[test]
fn test_multiboot2_cmdline_tag_parsing() {
    #[repr(C, align(8))]
    struct SyntheticMb2 {
        total_size: u32,
        reserved: u32,
        cmd_type: u32,
        cmd_size: u32,
        cmd_bytes: [u8; 32],
        end_type: u32,
        end_size: u32,
    }

    let mut mb2 = SyntheticMb2 {
        total_size: 56,
        reserved: 0,
        cmd_type: TAG_TYPE_CMDLINE,
        cmd_size: 40,
        cmd_bytes: [0u8; 32],
        end_type: TAG_TYPE_END,
        end_size: 8,
    };
    let text = b"console=ring0 init=/bin/sh\0";
    mb2.cmd_bytes[..text.len()].copy_from_slice(text);

    keira_core::reset_boot_cmdline();
    let ptr = (&raw const mb2) as usize;
    let _ = unsafe { parse_multiboot2(ptr) };

    assert!(keira_core::is_boot_cmdline_set());
    assert!(keira_core::has_boot_cmdline_flag("console=ring0"));
    let mut val = [0u8; 32];
    assert_eq!(
        keira_core::get_boot_cmdline_value("init", &mut val),
        Some("/bin/sh")
    );
    keira_core::reset_boot_cmdline();
}
