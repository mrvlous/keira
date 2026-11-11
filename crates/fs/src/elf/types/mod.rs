// SPDX-License-Identifier: GPL-2.0-only

//! Executable and Linkable Format (ELF) header and program header types.

pub mod header;
pub mod program;

pub use header::{Elf32Header, ElfHeader};
pub use program::{
    Program32Header, ProgramHeader, PF_R, PF_W, PF_X, PT_LOAD, USER_MAX_VADDR, USER_MIN_VADDR,
};
