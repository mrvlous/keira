// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

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
    assert_eq!(TAG_TYPE_MODULE, 3);
    assert_eq!(TAG_TYPE_FRAMEBUFFER, 8);
}
