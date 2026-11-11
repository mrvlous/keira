// SPDX-License-Identifier: GPL-2.0-only
/*
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra <mrvlous@proton.me>
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 */

//! Virtual Memory Area (VMA) management, demand paging and memory mapping system calls.

pub mod descriptor;
pub mod hooks;
pub mod invariants;
pub mod ops;
pub mod table;

pub use descriptor::{
    Vma, MAP_ANONYMOUS, MAP_FIXED, MAP_POPULATE, MAP_PRIVATE, MAP_SHARED, MAX_VMAS, MMAP_END,
    MMAP_START, MS_ASYNC, MS_INVALIDATE, MS_SYNC, PROT_EXEC, PROT_NONE, PROT_READ, PROT_WRITE,
    SUPPORTED_PROT,
};
pub use hooks::{
    get_file_read_hook, get_file_sync_hook, register_file_backing_hooks, FileReadHook, FileSyncHook,
};
pub use invariants::verify_vma_pte_invariants;
pub use ops::{
    madvise_pages, mmap_anonymous, mprotect_pages, munmap_pages, sys_mmap, sys_mmap_file,
    sys_mprotect, sys_msync, sys_munmap, sys_munmap_ext,
};
pub use table::{
    cleanup_vmas_for_pml4, clone_vmas_for_pml4, find_active_vma, find_free_mmap_range,
    get_vmas_for_pml4, register_mmap_aslr_hook, total_active_vmas, validate_virt_addr_range,
    VMA_TABLE,
};
