// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Directory and file inventory listing for in-memory Initrd archives.

use super::bounds::{INITRD_END, INITRD_START};
use crate::tar::header::{
    octal_str_to_u64, TYPEFLAG_DIRECTORY, TYPEFLAG_REGULAR, TYPEFLAG_REGULAR_ALT,
};
use keira_io::vga;

/// Prints formatted inventory of files and directories within the Initrd archive to VGA console.
pub fn list_files() {
    let mut addr = unsafe { INITRD_START };
    let end = unsafe { INITRD_END };
    if addr == 0 || end == 0 {
        vga::set_color(vga::Color::LightRed, vga::Color::Black);
        vga::print_str("Initrd not loaded.\n");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        return;
    }

    vga::set_color(vga::Color::White, vga::Color::Black);
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
            match core::str::from_utf8(core::slice::from_raw_parts(name_ptr, name_len)) {
                Ok(s) => s,
                Err(_) => "unknown",
            }
        };
        let typeflag = unsafe { *((addr + 156) as *const u8) };

        if typeflag == TYPEFLAG_REGULAR || typeflag == TYPEFLAG_REGULAR_ALT {
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            vga::print_str("  [file] ");
            vga::print_str(name);
            vga::print_str(" (");
            vga::print_u64(size);
            vga::print_str(" bytes)\n");
        } else if typeflag == TYPEFLAG_DIRECTORY {
            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("  [dir]  ");
            vga::print_str(name);
            vga::print_str("\n");
        }

        let blocks = (size + 511) / 512;
        addr += 512 + (blocks * 512);
    }
    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
}

/// Dumps textual content of an Initrd file to the VGA console display.
pub fn cat_file(path: &str) {
    let mut addr = unsafe { INITRD_START };
    let end = unsafe { INITRD_END };
    if addr == 0 || end == 0 {
        vga::set_color(vga::Color::LightRed, vga::Color::Black);
        vga::print_str("Initrd not loaded.\n");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        return;
    }

    let search_name = path.strip_prefix('/').unwrap_or(path);

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
            match core::str::from_utf8(core::slice::from_raw_parts(name_ptr, name_len)) {
                Ok(s) => s,
                Err(_) => "",
            }
        };

        if name == search_name || name.strip_prefix("./") == Some(search_name) {
            let data_ptr = (addr + 512) as *const u8;
            let slice = unsafe { core::slice::from_raw_parts(data_ptr, size as usize) };
            if let Ok(s) = core::str::from_utf8(slice) {
                vga::set_color(vga::Color::White, vga::Color::Black);
                vga::print_str(s);
                vga::print_str("\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            } else {
                vga::set_color(vga::Color::Yellow, vga::Color::Black);
                vga::print_str("Binary file (cannot display as UTF-8 string)\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            }
            return;
        }

        let blocks = (size + 511) / 512;
        addr += 512 + (blocks * 512);
    }

    vga::set_color(vga::Color::LightRed, vga::Color::Black);
    vga::print_str("File not found in Initrd: ");
    vga::print_str(path);
    vga::print_str("\n");
    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
}
