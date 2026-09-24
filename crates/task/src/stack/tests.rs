// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Unit tests for stack initialization and ELF auxiliary vector formatting.

use super::*;

#[test]
fn test_auxv_constants() {
    assert_eq!(AT_NULL, 0);
    assert_eq!(AT_PAGESZ, 6);
    assert_eq!(AT_ENTRY, 9);
    assert_eq!(AT_RANDOM, 25);
}

#[test]
fn test_user_stack_64_layout() {
    let mut page = [0u8; 4096];
    let top_vaddr = 0x7FFFFFE00000 - 4096;
    let args = ["/system/bin/test_abi.elf", "arg1", "arg2"];
    let rsp = unsafe { setup_user_stack_64(page.as_mut_ptr(), top_vaddr, &args, 0x400000) };
    assert!(rsp > top_vaddr);
    assert!(rsp < top_vaddr + 4096);
    assert_eq!(rsp % 16, 0);

    let offset = (rsp - top_vaddr) as usize;
    let argc = u64::from_le_bytes(page[offset..offset + 8].try_into().unwrap());
    assert_eq!(argc, 3);
}

#[test]
fn test_user_stack_32_layout() {
    let mut page = [0u8; 4096];
    let top_vaddr = 0xBFFF0000;
    let args = ["/bin/sh", "test"];
    let esp = unsafe { setup_user_stack_32(page.as_mut_ptr(), top_vaddr, &args, 0x08048000) };
    assert!(esp > top_vaddr);
    assert!(esp < top_vaddr + 4096);
}
