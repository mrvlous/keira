// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! 64-bit Task State Segment (TSS), RSP0 kernel privilege stack, and IST interrupt table setup.

use keira_mem::pmm;

#[repr(C, packed)]
pub struct TaskStateSegment {
    pub reserved0: u32,
    pub rsp0: u64,
    pub rsp1: u64,
    pub rsp2: u64,
    pub reserved1: u64,
    pub ist1: u64,
    pub ist2: u64,
    pub ist3: u64,
    pub ist4: u64,
    pub ist5: u64,
    pub ist6: u64,
    pub ist7: u64,
    pub reserved2: u64,
    pub reserved3: u16,
    pub iopb_offset: u16,
}

pub static mut TSS: TaskStateSegment = TaskStateSegment {
    reserved0: 0,
    rsp0: 0,
    rsp1: 0,
    rsp2: 0,
    reserved1: 0,
    ist1: 0,
    ist2: 0,
    ist3: 0,
    ist4: 0,
    ist5: 0,
    ist6: 0,
    ist7: 0,
    reserved2: 0,
    reserved3: 0,
    iopb_offset: 104,
};

extern "C" {
    static mut tss_descriptor: [u8; 16];
    fn reload_gdt();
    fn load_tss();
    fn init_syscall_msrs();
}

/// Initialize User Mode structures: populates GDT TSS entry, reloads GDT,
/// loads TSS register, and configures syscall MSR registers.
pub unsafe fn init_user_mode() {
    let tss_addr = &raw const TSS as u64;
    let tss_size = core::mem::size_of::<TaskStateSegment>() as u64 - 1;

    let stack_frame = pmm::alloc_frame().expect("TSS Init: Out of memory for RSP0 stack");
    TSS.rsp0 = stack_frame + pmm::PAGE_SIZE;

    let desc = &raw mut tss_descriptor;

    *(desc.cast::<u16>()) = tss_size as u16;
    *((desc as u64 + 2) as *mut u16) = (tss_addr & 0xFFFF) as u16;
    *((desc as u64 + 4) as *mut u8) = ((tss_addr >> 16) & 0xFF) as u8;
    *((desc as u64 + 5) as *mut u8) = 0x89;
    *((desc as u64 + 6) as *mut u8) = 0x00;
    *((desc as u64 + 7) as *mut u8) = ((tss_addr >> 24) & 0xFF) as u8;
    *((desc as u64 + 8) as *mut u32) = ((tss_addr >> 32) & 0xFFFFFFFF) as u32;
    *((desc as u64 + 12) as *mut u32) = 0;

    reload_gdt();
    load_tss();
    init_syscall_msrs();
}

/// Dynamically updates the TSS RSP0 stack pointer loaded when switching from Ring 3 to Ring 0.
pub unsafe fn set_kernel_stack(rsp0: u64) {
    TSS.rsp0 = rsp0;
}
