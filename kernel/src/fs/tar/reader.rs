// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//!
//! POSIX USTAR archive parser, in-memory initrd ramdisk unpacker, and file reader.

use crate::io::vga;

static mut INITRD_START: u64 = 0;
static mut INITRD_END: u64 = 0;

pub fn init(start: u64, end: u64) {
    unsafe {
        INITRD_START = start;
        INITRD_END = end;
    }
}

fn octal_str_to_u64(s: &[u8]) -> u64 {
    let mut res = 0;
    for &b in s {
        if b >= b'0' && b <= b'7' {
            res = (res << 3) | ((b - b'0') as u64);
        } else if b == 0 || b == b' ' {
            if res != 0 || b == 0 {}
        }
    }
    res
}

pub fn list_files() {
    let mut addr = unsafe { INITRD_START };
    let end = unsafe { INITRD_END };
    if addr == 0 || end == 0 {
        vga::set_color(vga::Color::LightRed, vga::Color::Black);
        vga::print_str("Initrd not loaded.\n");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        return;
    }

    vga::set_color(vga::Color::LightBlue, vga::Color::Black);
    vga::print_str("Files in Initrd:\n");
    while addr < end {
        let name_ptr = addr as *const u8;
        unsafe {
            if *name_ptr == 0 {
                break;
            }
        }

        let size_slice = unsafe { core::slice::from_raw_parts((addr + 124) as *const u8, 11) };
        let size = octal_str_to_u64(size_slice);

        let mut name_len = 0;
        unsafe {
            while name_len < 100 && *(name_ptr.add(name_len)) != 0 {
                name_len += 1;
            }
        }
        let name = unsafe {
            core::str::from_utf8_unchecked(core::slice::from_raw_parts(name_ptr, name_len))
        };
        let typeflag = unsafe { *((addr + 156) as *const u8) };

        if typeflag == b'0' || typeflag == 0 {
            vga::set_color(vga::Color::LightGreen, vga::Color::Black);
            vga::print_str("  [file] ");
            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str(name);

            vga::set_color(vga::Color::DarkGrey, vga::Color::Black);
            vga::print_str(" (");
            vga::print_u64(size);
            vga::print_str(" bytes)\n");
        } else if typeflag == b'5' {
            vga::set_color(vga::Color::LightBlue, vga::Color::Black);
            vga::print_str("  [dir]  ");
            vga::print_str(name);
            vga::print_str("\n");
        }

        addr += 512 + ((size + 511) / 512) * 512;
    }
    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
}

pub fn cat_file(target: &str) -> Result<(), &'static str> {
    let mut addr = unsafe { INITRD_START };
    let end = unsafe { INITRD_END };
    if addr == 0 || end == 0 {
        return Err("Initrd not loaded");
    }

    while addr < end {
        let name_ptr = addr as *const u8;
        unsafe {
            if *name_ptr == 0 {
                break;
            }
        }

        let size_slice = unsafe { core::slice::from_raw_parts((addr + 124) as *const u8, 11) };
        let size = octal_str_to_u64(size_slice);

        let mut name_len = 0;
        unsafe {
            while name_len < 100 && *(name_ptr.add(name_len)) != 0 {
                name_len += 1;
            }
        }
        let name = unsafe {
            core::str::from_utf8_unchecked(core::slice::from_raw_parts(name_ptr, name_len))
        };
        let typeflag = unsafe { *((addr + 156) as *const u8) };

        if name == target && (typeflag == b'0' || typeflag == 0) {
            let file_data =
                unsafe { core::slice::from_raw_parts((addr + 512) as *const u8, size as usize) };
            if let Ok(s) = core::str::from_utf8(file_data) {
                vga::print_str(s);
                if !s.ends_with('\n') {
                    vga::print_str("\n");
                }
            } else {
                vga::print_str("Error: File is not valid UTF-8 text.\n");
            }
            return Ok(());
        }

        addr += 512 + ((size + 511) / 512) * 512;
    }
    Err("File not found")
}

pub fn exists(target: &str) -> bool {
    let mut addr = unsafe { INITRD_START };
    let end = unsafe { INITRD_END };
    if addr == 0 || end == 0 {
        return false;
    }

    let search_target = if target.starts_with('/') {
        &target[1..]
    } else {
        target
    };

    while addr < end {
        let name_ptr = addr as *const u8;
        unsafe {
            if *name_ptr == 0 {
                break;
            }
        }

        let size_slice = unsafe { core::slice::from_raw_parts((addr + 124) as *const u8, 11) };
        let size = octal_str_to_u64(size_slice);

        let mut name_len = 0;
        unsafe {
            while name_len < 100 && *(name_ptr.add(name_len)) != 0 {
                name_len += 1;
            }
        }
        let name = unsafe {
            core::str::from_utf8_unchecked(core::slice::from_raw_parts(name_ptr, name_len))
        };
        let typeflag = unsafe { *((addr + 156) as *const u8) };

        let check_name = if name.starts_with('/') {
            &name[1..]
        } else {
            name
        };

        if check_name == search_target && (typeflag == b'0' || typeflag == 0) {
            return true;
        }

        addr += 512 + ((size + 511) / 512) * 512;
    }
    false
}

pub fn read_file_content(target: &str, buf: &mut [u8]) -> Result<usize, &'static str> {
    let mut addr = unsafe { INITRD_START };
    let end = unsafe { INITRD_END };
    if addr == 0 || end == 0 {
        return Err("Initrd not loaded");
    }

    let search_target = if target.starts_with('/') {
        &target[1..]
    } else {
        target
    };

    while addr < end {
        let name_ptr = addr as *const u8;
        unsafe {
            if *name_ptr == 0 {
                break;
            }
        }

        let size_slice = unsafe { core::slice::from_raw_parts((addr + 124) as *const u8, 11) };
        let size = octal_str_to_u64(size_slice);

        let mut name_len = 0;
        unsafe {
            while name_len < 100 && *(name_ptr.add(name_len)) != 0 {
                name_len += 1;
            }
        }
        let name = unsafe {
            core::str::from_utf8_unchecked(core::slice::from_raw_parts(name_ptr, name_len))
        };
        let typeflag = unsafe { *((addr + 156) as *const u8) };

        let check_name = if name.starts_with('/') {
            &name[1..]
        } else {
            name
        };

        if check_name == search_target && (typeflag == b'0' || typeflag == 0) {
            if size as usize > buf.len() {
                return Err("Buffer is too small for TAR file content");
            }
            let file_data =
                unsafe { core::slice::from_raw_parts((addr + 512) as *const u8, size as usize) };
            buf[..size as usize].copy_from_slice(file_data);
            return Ok(size as usize);
        }

        addr += 512 + ((size + 511) / 512) * 512;
    }
    Err("File not found in Initrd")
}
