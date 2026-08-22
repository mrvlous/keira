// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! 64-bit Executable and Linkable Format (ELF64) header and program header types.

/// 64-bit ELF file header.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ElfHeader {
    pub ident: [u8; 16],
    pub elf_type: u16,
    pub machine: u16,
    pub version: u32,
    pub entry: u64,
    pub phoff: u64,
    pub shoff: u64,
    pub flags: u32,
    pub ehsize: u16,
    pub phentsize: u16,
    pub phnum: u16,
    pub shentsize: u16,
    pub shnum: u16,
    pub shstrndx: u16,
}

/// 64-bit ELF program header segment descriptor.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ProgramHeader {
    pub p_type: u32,
    pub p_flags: u32,
    pub p_offset: u64,
    pub p_vaddr: u64,
    pub p_paddr: u64,
    pub p_filesz: u64,
    pub p_memsz: u64,
    pub p_align: u64,
}

pub const PT_LOAD: u32 = 1;

/// Program segment execute permission flag.
pub const PF_X: u32 = 1 << 0;
/// Program segment write permission flag.
pub const PF_W: u32 = 1 << 1;
/// Program segment read permission flag.
pub const PF_R: u32 = 1 << 2;

/// Canonical minimum user virtual address.
pub const USER_MIN_VADDR: u64 = 0x10000;
/// Canonical maximum user virtual address (lower-half canonical boundary).
pub const USER_MAX_VADDR: u64 = 0x0000_7FFF_FFFF_FFFF;
