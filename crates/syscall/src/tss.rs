// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Task State Segment (TSS), RSP0/ESP0 kernel privilege stack, and userland transition setup.

use keira_mem::pmm;

#[cfg(target_arch = "x86_64")]
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

#[cfg(target_arch = "x86")]
#[repr(C, packed)]
pub struct TaskStateSegment {
    pub prev_tss: u32,
    pub esp0: u32,
    pub ss0: u32,
    pub esp1: u32,
    pub ss1: u32,
    pub esp2: u32,
    pub ss2: u32,
    pub cr3: u32,
    pub eip: u32,
    pub eflags: u32,
    pub eax: u32,
    pub ecx: u32,
    pub edx: u32,
    pub ebx: u32,
    pub esp: u32,
    pub ebp: u32,
    pub esi: u32,
    pub edi: u32,
    pub es: u32,
    pub cs: u32,
    pub ss: u32,
    pub ds: u32,
    pub fs: u32,
    pub gs: u32,
    pub ldt: u32,
    pub trap: u16,
    pub iomap_base: u16,
}

#[cfg(target_arch = "x86_64")]
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

#[cfg(target_arch = "x86")]
pub static mut TSS: TaskStateSegment = TaskStateSegment {
    prev_tss: 0,
    esp0: 0,
    ss0: 0x10,
    esp1: 0,
    ss1: 0,
    esp2: 0,
    ss2: 0,
    cr3: 0,
    eip: 0,
    eflags: 0,
    eax: 0,
    ecx: 0,
    edx: 0,
    ebx: 0,
    esp: 0,
    ebp: 0,
    esi: 0,
    edi: 0,
    es: 0x10,
    cs: 0x08,
    ss: 0x10,
    ds: 0x10,
    fs: 0x10,
    gs: 0x10,
    ldt: 0,
    trap: 0,
    iomap_base: 104,
};

extern "C" {
    #[cfg(target_arch = "x86_64")]
    static mut tss_descriptor: [u8; 16];
    #[cfg(target_arch = "x86")]
    static mut tss_descriptor: [u8; 8];

    fn reload_gdt();
    fn load_tss();
    #[cfg(target_arch = "x86_64")]
    fn init_syscall_msrs();
}

/// Initialize User Mode structures: populates GDT TSS entry, reloads GDT,
/// loads TSS register, and configures syscall MSR registers.
pub unsafe fn init_user_mode() {
    let tss_addr = &raw const TSS as usize;
    let tss_size = core::mem::size_of::<TaskStateSegment>() - 1;

    let stack_frame =
        pmm::alloc_frame().expect("TSS Init: Out of memory for kernel privilege stack");

    #[cfg(target_arch = "x86_64")]
    {
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

    #[cfg(target_arch = "x86")]
    {
        TSS.esp0 = (stack_frame + pmm::PAGE_SIZE) as u32;
        TSS.ss0 = 0x10;

        let desc = &raw mut tss_descriptor;
        *(desc.cast::<u16>()) = tss_size as u16;
        *((desc as usize + 2) as *mut u16) = (tss_addr & 0xFFFF) as u16;
        *((desc as usize + 4) as *mut u8) = ((tss_addr >> 16) & 0xFF) as u8;
        *((desc as usize + 5) as *mut u8) = 0x89;
        *((desc as usize + 6) as *mut u8) = 0x00;
        *((desc as usize + 7) as *mut u8) = ((tss_addr >> 24) & 0xFF) as u8;

        reload_gdt();
        load_tss();
    }
}

/// Dynamically updates the TSS RSP0/ESP0 stack pointer loaded when switching from Ring 3 to Ring 0.
pub unsafe fn set_kernel_stack(sp0: usize) {
    #[cfg(target_arch = "x86_64")]
    {
        TSS.rsp0 = sp0 as u64;
    }
    #[cfg(target_arch = "x86")]
    {
        TSS.esp0 = sp0 as u32;
    }
}
