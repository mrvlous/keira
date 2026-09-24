// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Per-CPU hardware control blocks, dedicated kernel stacks, and MSR GS-base management.
//!
//! Maintains per-core execution state, interrupt stack tops, system call scratch registers,
//! and thread contexts ensuring thread isolation across Symmetric Multiprocessing (SMP) cores.

use crate::interrupts::smp::MAX_CORES;

/// Size of the dedicated kernel stack allocated per CPU core (16 KiB).
pub const PER_CPU_STACK_SIZE: usize = 16 * 1024;

/// Dedicated page-aligned kernel privilege stack allocated per CPU core.
#[repr(C, align(4096))]
pub struct KernelStack {
    pub stack: [u8; PER_CPU_STACK_SIZE],
}

/// Cache-line aligned (64-byte) control block for per-CPU hardware execution state.
///
/// Byte offsets are strictly mapped to low-level assembly in `arch/x86/x86_64/kernel/syscall.asm`:
/// - `0x00`: `self_ptr`
/// - `0x08`: `user_rsp_scratch`
/// - `0x10`: `kernel_stack`
/// - `0x18`: `main_stack`
/// - `0x20`: `core_id`
/// - `0x24`: `syscall_depth`
/// - `0x28`: `user_rip`
/// - `0x30`: `user_rflags`
/// - `0x38`: `user_rbx`
/// - `0x40`: `user_rbp`
/// - `0x48`: `user_r12`
/// - `0x50`: `user_r13`
/// - `0x58`: `user_r14`
/// - `0x60`: `user_r15`
/// - `0x68`: `user_rsp`
/// - `0x70`: `current_task_id`
#[repr(C, align(64))]
pub struct PerCpu {
    pub self_ptr: u64,
    pub user_rsp_scratch: u64,
    pub kernel_stack: u64,
    pub main_stack: u64,
    pub core_id: u32,
    pub syscall_depth: u32,
    pub user_rip: u64,
    pub user_rflags: u64,
    pub user_rbx: u64,
    pub user_rbp: u64,
    pub user_r12: u64,
    pub user_r13: u64,
    pub user_r14: u64,
    pub user_r15: u64,
    pub user_rsp: u64,
    pub current_task_id: u64,
    pub reserved: [u64; 1],
}

/// Global dedicated kernel privilege stacks for each SMP CPU core.
pub static mut PER_CPU_STACKS: [KernelStack; MAX_CORES] = [const {
    KernelStack {
        stack: [0u8; PER_CPU_STACK_SIZE],
    }
}; MAX_CORES];

/// Global array of per-CPU control blocks.
pub static mut PER_CPU_DATA: [PerCpu; MAX_CORES] = [const {
    PerCpu {
        self_ptr: 0,
        user_rsp_scratch: 0,
        kernel_stack: 0,
        main_stack: 0,
        core_id: 0,
        syscall_depth: 0,
        user_rip: 0,
        user_rflags: 0x202,
        user_rbx: 0,
        user_rbp: 0,
        user_r12: 0,
        user_r13: 0,
        user_r14: 0,
        user_r15: 0,
        user_rsp: 0,
        current_task_id: 0,
        reserved: [0; 1],
    }
}; MAX_CORES];

/// Initializes Per-CPU control block fields and stack top pointers for the specified core.
///
/// # Safety
///
/// Must be invoked during early bootstrap or SMP core initialization before enabling interrupts.
pub unsafe fn init_percpu_data(core_id: usize) {
    if core_id >= MAX_CORES {
        return;
    }

    let percpu_ptr = &raw mut PER_CPU_DATA[core_id];
    let stack_top = (&raw const PER_CPU_STACKS[core_id].stack as usize + PER_CPU_STACK_SIZE) as u64;

    (*percpu_ptr).self_ptr = percpu_ptr as u64;
    (*percpu_ptr).user_rsp_scratch = 0;
    (*percpu_ptr).kernel_stack = stack_top;
    (*percpu_ptr).main_stack = stack_top;
    (*percpu_ptr).core_id = core_id as u32;
    (*percpu_ptr).syscall_depth = 0;
    (*percpu_ptr).user_rip = 0;
    (*percpu_ptr).user_rflags = 0x202;
    (*percpu_ptr).user_rbx = 0;
    (*percpu_ptr).user_rbp = 0;
    (*percpu_ptr).user_r12 = 0;
    (*percpu_ptr).user_r13 = 0;
    (*percpu_ptr).user_r14 = 0;
    (*percpu_ptr).user_r15 = 0;
    (*percpu_ptr).user_rsp = 0;
    (*percpu_ptr).current_task_id = 0;
    (*percpu_ptr).reserved = [0; 1];
}

/// Loads GS base Model-Specific Registers for the calling CPU core.
///
/// # Safety
///
/// Must only be executed by the target core on itself.
pub unsafe fn load_percpu_msrs(core_id: usize) {
    if core_id >= MAX_CORES {
        return;
    }
    #[cfg(all(target_arch = "x86_64", target_os = "none"))]
    {
        let percpu_ptr = &raw mut PER_CPU_DATA[core_id];
        crate::cpu::control::msr::wrmsr(
            crate::cpu::control::msr::IA32_GS_BASE_MSR,
            percpu_ptr as u64,
        );
        crate::cpu::control::msr::wrmsr(crate::cpu::control::msr::IA32_KERNEL_GS_BASE_MSR, 0);
    }
}

/// Initializes Per-CPU data structure, stack top, and MSR registers for the current core.
///
/// # Safety
///
/// Must be invoked during CPU core bootstrap with interrupts disabled.
pub unsafe fn init_percpu(core_id: usize) {
    init_percpu_data(core_id);
    load_percpu_msrs(core_id);
}

/// Retrieves the hardware or logical ID of the calling CPU core.
pub fn get_current_core_id() -> usize {
    #[cfg(target_os = "none")]
    unsafe {
        let lapic_id = (crate::interrupts::apic::get_current_lapic_id() & 0xFF) as u8;
        for i in 0..MAX_CORES {
            if let Some(core) = crate::interrupts::smp::SMP_CORES[i] {
                if core.apic_id == lapic_id {
                    return core.core_id as usize;
                }
            }
        }
        0
    }
    #[cfg(not(target_os = "none"))]
    {
        0
    }
}

/// Retrieves a mutable reference to the current CPU core's `PerCpu` control block.
#[inline(always)]
pub fn get_current_percpu() -> &'static mut PerCpu {
    let core_id = get_current_core_id();
    unsafe { &mut PER_CPU_DATA[core_id] }
}

/// Retrieves a mutable reference to a specific CPU core's `PerCpu` control block.
pub fn get_percpu(core_id: usize) -> Option<&'static mut PerCpu> {
    if core_id < MAX_CORES {
        unsafe { Some(&mut PER_CPU_DATA[core_id]) }
    } else {
        None
    }
}

/// Updates the kernel stack top for the current CPU core.
///
/// # Safety
///
/// Must be called during scheduler task switching or syscall stack configuration.
pub unsafe fn set_current_kernel_stack(stack: u64) {
    let core_id = get_current_core_id();
    if core_id < MAX_CORES {
        PER_CPU_DATA[core_id].kernel_stack = stack;
    }
}

/// Retrieves the active kernel stack top for the current CPU core.
pub fn get_current_kernel_stack() -> u64 {
    let core_id = get_current_core_id();
    unsafe { PER_CPU_DATA[core_id].kernel_stack }
}
