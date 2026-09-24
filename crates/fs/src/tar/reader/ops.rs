// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Archive queries, file size calculation, and memory-backed reading operations.

use super::bounds::{INITRD_END, INITRD_START};
use crate::tar::header::octal_str_to_u64;

/// Checks whether a given path exists in the loaded Initrd archive.
pub fn exists(path: &str) -> bool {
    let mut addr = unsafe { INITRD_START };
    let end = unsafe { INITRD_END };
    if addr == 0 || end == 0 {
        return false;
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
            return true;
        }

        let blocks = (size + 511) / 512;
        addr += 512 + (blocks * 512);
    }

    false
}

/// Retrieves the size in bytes of a file stored in the Initrd archive.
pub fn get_file_size(path: &str) -> Result<usize, &'static str> {
    let mut addr = unsafe { INITRD_START };
    let end = unsafe { INITRD_END };
    if addr == 0 || end == 0 {
        return Err("Initrd not loaded");
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
            return Ok(size as usize);
        }

        let blocks = (size + 511) / 512;
        addr += 512 + (blocks * 512);
    }

    Err("File not found in Initrd")
}

/// Reads file content from Initrd into a caller-supplied buffer.
pub fn read_file_content(path: &str, buf: &mut [u8]) -> Result<usize, &'static str> {
    let mut addr = unsafe { INITRD_START };
    let end = unsafe { INITRD_END };
    if addr == 0 || end == 0 {
        return Err("Initrd not loaded");
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
            let read_len = core::cmp::min(size as usize, buf.len());
            unsafe {
                core::ptr::copy_nonoverlapping(data_ptr, buf.as_mut_ptr(), read_len);
            }
            return Ok(read_len);
        }

        let blocks = (size + 511) / 512;
        addr += 512 + (blocks * 512);
    }

    Err("File not found in Initrd")
}

/// Reads file content starting at a specific byte offset from Initrd into buffer.
pub fn read_file_offset(path: &str, offset: u64, buf: &mut [u8]) -> Result<usize, &'static str> {
    let mut addr = unsafe { INITRD_START };
    let end = unsafe { INITRD_END };
    if addr == 0 || end == 0 {
        return Err("Initrd not loaded");
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
            if offset >= size {
                return Ok(0);
            }
            let data_ptr = (addr + 512 + offset) as *const u8;
            let avail = (size - offset) as usize;
            let read_len = core::cmp::min(avail, buf.len());
            unsafe {
                core::ptr::copy_nonoverlapping(data_ptr, buf.as_mut_ptr(), read_len);
            }
            return Ok(read_len);
        }

        let blocks = (size + 511) / 512;
        addr += 512 + (blocks * 512);
    }

    Err("File not found in Initrd")
}
