// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! System V AMD64 ABI user stack layout formatting.

use crate::stack::auxv::*;

/// Format System V AMD64 ABI user stack with argc, argv pointers, envp pointers, auxv, and string data.
///
/// # Safety
/// The caller must ensure that `page_ptr` points to at least 4096 bytes of writable mapped memory.
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
