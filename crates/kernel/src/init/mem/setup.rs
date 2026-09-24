// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Memory management, paging, heap, and linear framebuffer initialization.

use keira_io::ps2::mouse as ps2_mouse;
use keira_io::vga;
use keira_mem::vmm;

#[cfg(target_os = "none")]
extern "C" {
    static __bss_end: u8;
    static stack_top: u8;
}

#[cfg(not(target_os = "none"))]
#[allow(non_upper_case_globals)]
static __bss_end: u8 = 0;
#[cfg(not(target_os = "none"))]
#[allow(non_upper_case_globals)]
static stack_top: u8 = 0;

/// Initialize Physical Frame Allocator (PMM), Paging (VMM), and Segregated Free-List Heap.
///
/// # Safety
/// Caller must ensure `multiboot_info_ptr` is valid and mapped.
pub unsafe fn init_memory(multiboot_info_ptr: usize, initrd_end: u64) {
    let bss_end_addr = core::ptr::addr_of!(__bss_end) as u64;
    let stack_top_addr = core::ptr::addr_of!(stack_top) as u64;
    let heap_end_addr = core::cmp::max(bss_end_addr, stack_top_addr);

    keira_mem::init(multiboot_info_ptr as u64, initrd_end, heap_end_addr);

    let fb_addr = vga::FRAMEBUFFER_ADDR;
    let fb_pitch = vga::FRAMEBUFFER_PITCH;
    let fb_height = vga::FRAMEBUFFER_HEIGHT;
    if fb_addr != 0 {
        let fb_size = fb_height as u64 * fb_pitch as u64;
        let page_count = fb_size.div_ceil(4096);
        for i in 0..page_count {
            let offset = i * 4096;
            let phys = fb_addr + offset;
            let _ = vmm::map_page(phys, phys, vmm::PAGE_WRITABLE);
        }
        let fb_width = vga::FRAMEBUFFER_WIDTH;
        ps2_mouse::set_resolution(fb_width as i32, fb_height as i32);
        vga::FRAMEBUFFER_MAPPED = true;
        vga::init();
    }

    vga::print_boot_log("Initializing Physical Memory Manager (PMM) frames", 0);
    vga::print_boot_log("Initializing Virtual Memory Manager (VMM) paging", 0);
    vga::print_boot_log("Initializing Segregated Free-List Kernel Heap", 0);
}
