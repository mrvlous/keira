// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Top-level initialization routine for user mode transition structures.

use keira_mem::pmm;

use crate::tss::privilege::gdt::configure_gdt_tss;
use crate::tss::privilege::msr::configure_syscall_msrs;
use crate::tss::segment::stack::{BOOT_KERNEL_STACK_TOP, TSS};

/// Initializes User Mode structures: populates GDT TSS entry, reloads GDT,
/// loads TSS register, and configures syscall MSR registers.
pub unsafe fn init_user_mode() {
    keira_task::scheduler::register_task_cleanup_hook(crate::dispatcher::syscall_task_cleanup_hook);

    let stack_frame = pmm::alloc_frame().unwrap_or(0x1000_0000);
    BOOT_KERNEL_STACK_TOP = (stack_frame + pmm::PAGE_SIZE) as usize;

    #[cfg(target_arch = "x86_64")]
    {
        TSS.rsp0 = BOOT_KERNEL_STACK_TOP as u64;
    }

    #[cfg(target_arch = "x86")]
    {
        TSS.esp0 = BOOT_KERNEL_STACK_TOP as u32;
        TSS.ss0 = 0x10;
    }

    configure_gdt_tss();
    configure_syscall_msrs(BOOT_KERNEL_STACK_TOP);
}
