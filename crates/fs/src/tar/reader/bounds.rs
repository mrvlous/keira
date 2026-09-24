// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Memory boundaries configuration for loaded in-memory USTAR initrd ramdisk.

/// Starting physical memory address of the loaded initrd image.
pub static mut INITRD_START: u64 = 0;

/// Ending physical memory address of the loaded initrd image.
pub static mut INITRD_END: u64 = 0;

/// Initializes the in-memory UStar Initrd ramdisk boundary pointers.
pub fn init(start: u64, end: u64) {
    unsafe {
        INITRD_START = start;
        INITRD_END = end;
    }
}
