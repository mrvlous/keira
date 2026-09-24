// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Unit tests for ELF binary structures and loading invariants.

use super::loader::mapping::SegmentMapping;
use super::types::{
    Elf32Header, ElfHeader, Program32Header, ProgramHeader, PF_R, PF_W, PF_X, PT_LOAD,
    USER_MAX_VADDR, USER_MIN_VADDR,
};

#[test]
fn test_elf_header_sizes() {
    assert_eq!(core::mem::size_of::<ElfHeader>(), 64);
    assert_eq!(core::mem::size_of::<Elf32Header>(), 52);
    assert_eq!(core::mem::size_of::<ProgramHeader>(), 56);
    assert_eq!(core::mem::size_of::<Program32Header>(), 32);
}

#[test]
fn test_elf_flags_and_constants() {
    assert_eq!(PT_LOAD, 1);
    assert_eq!(PF_X, 1);
    assert_eq!(PF_W, 2);
    assert_eq!(PF_R, 4);

    let wx = PF_W | PF_X;
    assert!((wx & PF_W) != 0 && (wx & PF_X) != 0);

    let ro_x = PF_R | PF_X;
    assert!((ro_x & PF_W) == 0 && (ro_x & PF_X) != 0);
}

#[test]
fn test_user_canonical_boundaries() {
    assert!(USER_MIN_VADDR < USER_MAX_VADDR);
}

#[test]
fn test_empty_segment_mapping() {
    let empty = SegmentMapping::empty();
    assert_eq!(empty.aligned_start, 0);
    assert_eq!(empty.aligned_end, 0);
    assert_eq!(empty.mapped_bytes, 0);
    assert!(!empty.is_executable);
}
