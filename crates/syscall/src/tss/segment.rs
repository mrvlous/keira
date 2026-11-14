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

//! Architecture-specific Task State Segment (TSS) data structures and kernel stack control.

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

pub static mut BOOT_KERNEL_STACK_TOP: usize = 0;

/// Dynamically updates the TSS RSP0/ESP0 stack pointer and Per-CPU kernel stack
/// loaded when transitioning from Ring 3 to Ring 0.
///
/// # Safety
/// Caller must ensure `sp0` points to a valid, mapped and aligned kernel stack memory region.
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
///
/// # Safety
/// Accesses global bootstrap kernel stack state.
#[no_mangle]
pub unsafe extern "C" fn get_boot_kernel_stack() -> usize {
    BOOT_KERNEL_STACK_TOP
}

