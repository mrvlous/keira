// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! System V ABI initial user stack setup, argument formatting, and ELF auxiliary vector construction.

pub const AT_NULL: u64 = 0;
pub const AT_IGNORE: u64 = 1;
pub const AT_EXECFD: u64 = 2;
pub const AT_PHDR: u64 = 3;
pub const AT_PHENT: u64 = 4;
pub const AT_PHNUM: u64 = 5;
pub const AT_PAGESZ: u64 = 6;
pub const AT_BASE: u64 = 7;
pub const AT_FLAGS: u64 = 8;
pub const AT_ENTRY: u64 = 9;
pub const AT_NOTELF: u64 = 10;
pub const AT_UID: u64 = 11;
pub const AT_EUID: u64 = 12;
pub const AT_GID: u64 = 13;
pub const AT_EGID: u64 = 14;
pub const AT_CLKTCK: u64 = 17;
pub const AT_RANDOM: u64 = 25;
pub const AT_EXECFN: u64 = 31;

/// Format System V AMD64 ABI user stack with argc, argv pointers, envp pointers, auxv, and string data.
///
/// # Safety
/// The caller must ensure that `page_ptr` points to at least 4096 bytes of writable mapped memory.
#[cfg(target_arch = "x86_64")]
pub unsafe fn setup_user_stack_64(
    page_ptr: *mut u8,
    top_page_vaddr: u64,
    args: &[&str],
    entry_point: u64,
) -> u64 {
    let mut offset = 4096usize - 16;
    let mut arg_vaddrs = [0u64; 16];
    let argc = if args.is_empty() {
        1
    } else {
        args.len().min(16)
    };

    for (i, &arg) in args[..argc].iter().enumerate() {
        let bytes = arg.as_bytes();
        let len = bytes.len() + 1;
        if offset < len + 256 {
            break;
        }
        offset -= len;
        core::ptr::copy_nonoverlapping(bytes.as_ptr(), page_ptr.add(offset), bytes.len());
        *page_ptr.add(offset + bytes.len()) = 0;
        arg_vaddrs[i] = top_page_vaddr + offset as u64;
    }

    // Allocate 16 random entropy bytes for AT_RANDOM stack canary seed
    offset = offset.saturating_sub(16);
    let mut random_entropy = [
        0x4bu8, 0x65, 0x69, 0x72, 0x61, 0x5f, 0x72, 0x6e, 0x67, 0x5f, 0x73, 0x65, 0x65, 0x64, 0x32,
        0x36,
    ];
    let lo: u32;
    let hi: u32;
    core::arch::asm!("rdtsc", out("eax") lo, out("edx") hi);
    let tsc = ((hi as u64) << 32) | (lo as u64);
    for (i, byte) in random_entropy.iter_mut().enumerate() {
        *byte = ((tsc >> ((i % 8) * 8)) ^ (i as u64 * 0x9E37_79B9) ^ 0xA5) as u8;
    }
    if random_entropy[0] == 0 {
        random_entropy[0] = 0x4B;
    }
    core::ptr::copy_nonoverlapping(random_entropy.as_ptr(), page_ptr.add(offset), 16);
    let random_vaddr = top_page_vaddr + offset as u64;

    // 16-byte align before pointer words
    offset &= !15;

    let auxv: [(u64, u64); 8] = [
        (AT_PAGESZ, 4096),
        (AT_ENTRY, entry_point),
        (AT_BASE, 0),
        (AT_FLAGS, 0),
        (AT_UID, 0),
        (AT_EUID, 0),
        (AT_CLKTCK, 100),
        (AT_RANDOM, random_vaddr),
    ];
    let auxv_words = (auxv.len() + 1) * 2;

    // Total words: argc(1) + argv pointers(argc) + argv_null(1) + envp_null(1) + auxv_words
    let total_words = 1 + argc + 1 + 1 + auxv_words;
    if total_words % 2 != 0 {
        offset = offset.saturating_sub(8);
    }
    offset = offset.saturating_sub(total_words * 8);

    let stack_u64 = page_ptr.add(offset) as *mut u64;
    let mut w_idx = 0;

    // 1. argc
    *stack_u64.add(w_idx) = argc as u64;
    w_idx += 1;

    // 2. argv[0..argc]
    for i in 0..argc {
        *stack_u64.add(w_idx) = arg_vaddrs[i];
        w_idx += 1;
    }

    // 3. argv NULL terminator
    *stack_u64.add(w_idx) = 0;
    w_idx += 1;

    // 4. envp NULL terminator
    *stack_u64.add(w_idx) = 0;
    w_idx += 1;

    // 5. auxv table
    for (tag, val) in auxv.iter() {
        *stack_u64.add(w_idx) = *tag;
        *stack_u64.add(w_idx + 1) = *val;
        w_idx += 2;
    }
    *stack_u64.add(w_idx) = AT_NULL;
    *stack_u64.add(w_idx + 1) = 0;

    top_page_vaddr + offset as u64
}

/// Format 32-bit user stack with argc, argv pointers, envp pointers, auxv, and string data.
///
/// # Safety
/// The caller must ensure that `page_ptr` points to at least 4096 bytes of writable mapped memory.
#[cfg(target_arch = "x86")]
pub unsafe fn setup_user_stack_32(
    page_ptr: *mut u8,
    top_page_vaddr: u64,
    args: &[&str],
    entry_point: u64,
) -> u64 {
    let mut offset = 4096usize - 16;
    let mut arg_vaddrs = [0u32; 16];
    let argc = if args.is_empty() {
        1
    } else {
        args.len().min(16)
    };

    for (i, &arg) in args[..argc].iter().enumerate() {
        let bytes = arg.as_bytes();
        let len = bytes.len() + 1;
        if offset < len + 256 {
            break;
        }
        offset -= len;
        core::ptr::copy_nonoverlapping(bytes.as_ptr(), page_ptr.add(offset), bytes.len());
        *page_ptr.add(offset + bytes.len()) = 0;
        arg_vaddrs[i] = (top_page_vaddr + offset as u64) as u32;
    }

    // Allocate 16 random entropy bytes for AT_RANDOM stack canary seed
    offset = offset.saturating_sub(16);
    let mut random_entropy = [
        0x4bu8, 0x65, 0x69, 0x72, 0x61, 0x5f, 0x72, 0x6e, 0x67, 0x5f, 0x73, 0x65, 0x65, 0x64, 0x32,
        0x36,
    ];
    let lo: u32;
    let hi: u32;
    core::arch::asm!("rdtsc", out("eax") lo, out("edx") hi);
    let tsc = ((hi as u64) << 32) | (lo as u64);
    for (i, byte) in random_entropy.iter_mut().enumerate() {
        *byte = ((tsc >> ((i % 8) * 8)) ^ (i as u64 * 0x9E37_79B9) ^ 0xA5) as u8;
    }
    if random_entropy[0] == 0 {
        random_entropy[0] = 0x4B;
    }
    core::ptr::copy_nonoverlapping(random_entropy.as_ptr(), page_ptr.add(offset), 16);
    let random_vaddr = (top_page_vaddr + offset as u64) as u32;

    offset &= !15;

    let auxv: [(u32, u32); 8] = [
        (AT_PAGESZ as u32, 4096),
        (AT_ENTRY as u32, entry_point as u32),
        (AT_BASE as u32, 0),
        (AT_FLAGS as u32, 0),
        (AT_UID as u32, 0),
        (AT_EUID as u32, 0),
        (AT_CLKTCK as u32, 100),
        (AT_RANDOM as u32, random_vaddr),
    ];
    let auxv_words = (auxv.len() + 1) * 2;

    // Allocate argv vector array [argv[0], ..., argv[argc-1], NULL, envp_null(1), auxv]
    let table_words = (argc + 1) + 1 + auxv_words;
    offset = offset.saturating_sub(table_words * 4);
    let argv_table_offset = offset;
    let argv_table_ptr = page_ptr.add(argv_table_offset) as *mut u32;

    let mut w_idx = 0;
    for i in 0..argc {
        *argv_table_ptr.add(w_idx) = arg_vaddrs[i];
        w_idx += 1;
    }
    *argv_table_ptr.add(w_idx) = 0;
    w_idx += 1;
    *argv_table_ptr.add(w_idx) = 0;
    w_idx += 1;
    for (tag, val) in auxv.iter() {
        *argv_table_ptr.add(w_idx) = *tag;
        *argv_table_ptr.add(w_idx + 1) = *val;
        w_idx += 2;
    }
    *argv_table_ptr.add(w_idx) = AT_NULL as u32;
    *argv_table_ptr.add(w_idx + 1) = 0;

    // Stack pointer for i686 CDECL initial entry
    offset = offset.saturating_sub(16);
    let call_args = page_ptr.add(offset) as *mut u32;
    *call_args = 0; // Return address sentinel
    *call_args.add(1) = argc as u32;
    *call_args.add(2) = (top_page_vaddr + argv_table_offset as u64) as u32;
    *call_args.add(3) = (top_page_vaddr + (argv_table_offset + (argc + 1) * 4) as u64) as u32;

    top_page_vaddr + offset as u64
}
