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
