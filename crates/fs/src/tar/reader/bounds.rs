// SPDX-License-Identifier: GPL-2.0-only

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
