// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Keira Kernel: Memory Management Module Root
//!
//! Provides Physical Memory Management (PMM) and Virtual Memory Management (VMM).

pub mod dma;
pub mod kasan;
pub mod kfence;
pub mod kpti;
pub mod pmm;
pub mod swap;
pub mod vmm;

/// Initialize the PMM and VMM subsystems.
///
/// # Safety
/// This function executes unsafe operations like traversing Multiboot2 tags and writing page tables.
pub unsafe fn init(multiboot_info_ptr: u64, initrd_end: u64, heap_end: u64) {
    // Find the end of allocated kernel memory.
    // The kernel is loaded starting at 1MB, and initrd or heap may sit above it.
    // We pick the maximum end address as the safe boundary for free physical memory.
    let kernel_end = if initrd_end > heap_end {
        initrd_end
    } else {
        heap_end
    };

    pmm::init(multiboot_info_ptr, kernel_end);
}
