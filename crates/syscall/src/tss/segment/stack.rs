// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Global TSS instance and privilege level 0 kernel stack control.

use crate::tss::segment::types::TaskStateSegment;

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

pub static mut BOOT_KERNEL_STACK_TOP: usize = 0;

/// Dynamically updates the TSS RSP0/ESP0 stack pointer and Per-CPU kernel stack
/// loaded when transitioning from Ring 3 to Ring 0.
#[no_mangle]
pub unsafe extern "C" fn set_kernel_stack(sp0: usize) {
    #[cfg(target_arch = "x86_64")]
    {
        TSS.rsp0 = sp0 as u64;
        keira_arch::cpu::percpu::set_current_kernel_stack(sp0 as u64);
    }
    #[cfg(target_arch = "x86")]
    {
        TSS.esp0 = sp0 as u32;
    }
}

/// Retrieves the initial privilege stack top allocated at bootstrap.
#[no_mangle]
pub unsafe extern "C" fn get_boot_kernel_stack() -> usize {
    BOOT_KERNEL_STACK_TOP
}
