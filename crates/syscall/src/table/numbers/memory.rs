// SPDX-License-Identifier: GPL-2.0-only

//! Virtual memory and heap system call numbers.

pub const SYS_SBRK: u64 = 11;
pub const SYS_BRK: u64 = 12;
pub const SYS_MMAP: u64 = 20;
pub const SYS_MUNMAP: u64 = 21;
pub const SYS_MPROTECT: u64 = 31;
pub const SYS_MADVISE: u64 = 32;
pub const SYS_MSYNC: u64 = 83;
