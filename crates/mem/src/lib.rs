// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

#![no_std]

//! Physical memory management, 4-level virtual paging, DMA, and swap for Keira Kernel.

pub mod dma;
pub mod heap;
pub mod pmm;
pub mod swap;
pub mod vmm;

pub use dma::{alloc_dma_buffer, DmaBuffer, ScatterGatherEntry};
pub use heap::{
    heap_get_alloc_count, heap_get_free, heap_get_peak, heap_get_total, heap_get_used, heap_init,
    kfree, kmalloc,
};
pub use pmm::{
    alloc_frame, free_contiguous_frames, free_frame, get_stats, init as pmm_init,
    is_valid_ram_range, KERNEL_BASE_1MB, PAGE_SIZE, PAGE_SIZE_4K,
};
pub use swap::pager as swap_pager;
pub use swap::{is_active as swap_is_active, swapoff, swapon, sys_swapoff, sys_swapon};
pub use vmm::{
    active_pml4, cleanup_vmas_for_pml4, clone_kernel_pml4, find_free_mmap_range,
    free_and_unmap_page, free_user_pages, get_kaslr_offset, get_phys_addr, get_phys_addr_in_pml4,
    get_pte_in_pml4, is_page_mapped_in_pml4, madvise_pages, map_page, mmap_anonymous,
    mprotect_pages, munmap_pages, switch_address_space, sys_mmap, sys_mprotect, sys_munmap,
    sys_munmap_ext, translate_pte_to_phys, unmap_page, validate_virt_addr_range,
    verify_vma_pte_invariants, GB_1_IDENTITY_MAP, PAGE_HUGE, PAGE_NO_EXECUTE, PAGE_PRESENT,
    PAGE_USER, PAGE_WRITABLE, PTE_ADDR_MASK, PTE_ADDR_MASK_1G, PTE_ADDR_MASK_2M, PTE_ADDR_MASK_4K,
    USER_MAX_VADDR, USER_MIN_VADDR,
};

/// Initialize the PMM and VMM subsystems.
pub unsafe fn init(multiboot_info_ptr: u64, initrd_end: u64, heap_end: u64) {
    let kernel_end = if initrd_end > heap_end {
        initrd_end
    } else {
        heap_end
    };

    pmm::init(multiboot_info_ptr, kernel_end);
}
