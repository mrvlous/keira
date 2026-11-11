// SPDX-License-Identifier: GPL-2.0-only

//! Program header structures, segment flags and user virtual address bounds.

pub use keira_mem::vmm::{USER_MAX_VADDR, USER_MIN_VADDR};

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

/// 32-bit ELF program header segment descriptor.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Program32Header {
    pub p_type: u32,
    pub p_offset: u32,
    pub p_vaddr: u32,
    pub p_paddr: u32,
    pub p_filesz: u32,
    pub p_memsz: u32,
    pub p_flags: u32,
    pub p_align: u32,
}

/// Loadable program segment type.
pub const PT_LOAD: u32 = 1;

/// Program segment execute permission flag.
pub const PF_X: u32 = 1 << 0;

/// Program segment write permission flag.
pub const PF_W: u32 = 1 << 1;

/// Program segment read permission flag.
pub const PF_R: u32 = 1 << 2;
