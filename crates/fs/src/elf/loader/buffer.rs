// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Static staging buffer for loaded ELF binary images.

pub static mut ELF_FILE_BUF: [u8; 524288] = [0u8; 524288];
pub static mut LAST_LOADED_LEN: usize = 0;

/// Retrieve a slice of the raw ELF binary image loaded during the most recent execution.
///
/// # Safety
/// The caller must ensure no concurrent ELF loading occurs while reading the slice.
pub unsafe fn last_loaded_elf_slice() -> &'static [u8] {
    let buf_ptr = core::ptr::addr_of!(ELF_FILE_BUF) as *const u8;
    let len = LAST_LOADED_LEN.min(524288);
    core::slice::from_raw_parts(buf_ptr, len)
}
