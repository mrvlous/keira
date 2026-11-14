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

//! GDT descriptor programming, MSR configuration and user mode transition setup.

use keira_mem::pmm;

use crate::tss::segment::{TaskStateSegment, BOOT_KERNEL_STACK_TOP, TSS};

#[cfg(not(test))]
extern "C" {
    #[cfg(target_arch = "x86_64")]
    static mut tss_descriptor: [u8; 16];
    #[cfg(target_arch = "x86")]
    static mut tss_descriptor: [u8; 8];

    fn reload_gdt();
    fn load_tss();
}

#[cfg(test)]
#[repr(align(16))]
struct TestTssDescriptor([u8; 16]);

#[cfg(test)]
static mut TEST_TSS_DESCRIPTOR: TestTssDescriptor = TestTssDescriptor([0; 16]);

#[cfg(test)]
unsafe fn reload_gdt() {}

#[cfg(test)]
unsafe fn load_tss() {}

/// Populates the GDT TSS descriptor entry and flushes processor segment registers.
///
/// # Safety
/// Modifies low-level Global Descriptor Table descriptor entries and executes segment reload instructions.
pub unsafe fn configure_gdt_tss() {
    let tss_addr = &raw const TSS as usize;
    let tss_size = core::mem::size_of::<TaskStateSegment>() - 1;

    #[cfg(not(test))]
    let desc = (&raw mut tss_descriptor) as *mut u8;
    #[cfg(test)]
    let desc = core::ptr::addr_of_mut!(TEST_TSS_DESCRIPTOR.0).cast::<u8>();

    #[cfg(target_arch = "x86_64")]
    {
        core::ptr::write_unaligned(desc.cast::<u16>(), tss_size as u16);
        core::ptr::write_unaligned((desc.add(2)).cast::<u16>(), (tss_addr & 0xFFFF) as u16);
        core::ptr::write_unaligned(desc.add(4), ((tss_addr >> 16) & 0xFF) as u8);
        core::ptr::write_unaligned(desc.add(5), 0x89);
        core::ptr::write_unaligned(desc.add(6), 0x00);
        core::ptr::write_unaligned(desc.add(7), ((tss_addr >> 24) & 0xFF) as u8);
        core::ptr::write_unaligned(
            (desc.add(8)).cast::<u32>(),
            ((tss_addr >> 32) & 0xFFFFFFFF) as u32,
        );
        core::ptr::write_unaligned((desc.add(12)).cast::<u32>(), 0);

        reload_gdt();
        load_tss();
    }

    #[cfg(target_arch = "x86")]
    {
        core::ptr::write_unaligned(desc.cast::<u16>(), tss_size as u16);
        core::ptr::write_unaligned((desc.add(2)).cast::<u16>(), (tss_addr & 0xFFFF) as u16);
        core::ptr::write_unaligned(desc.add(4), ((tss_addr >> 16) & 0xFF) as u8);
        core::ptr::write_unaligned(desc.add(5), 0x89);
        core::ptr::write_unaligned(desc.add(6), 0x00);
        core::ptr::write_unaligned(desc.add(7), ((tss_addr >> 24) & 0xFF) as u8);

        reload_gdt();
        load_tss();
    }
}

#[cfg(all(target_arch = "x86_64", not(test)))]
extern "C" {
    fn init_syscall_msrs();
}

#[cfg(all(target_arch = "x86_64", test))]
unsafe fn init_syscall_msrs() {}

/// Configures syscall/sysret MSRs (STAR, LSTAR, FMASK) and per-cpu stacks.
///
/// # Safety
/// Writes model-specific registers (MSRs) and per-CPU kernel stack pointers directly.
pub unsafe fn configure_syscall_msrs(stack_top: usize) {
    #[cfg(target_arch = "x86_64")]
    {
        init_syscall_msrs();
        #[cfg(not(test))]
        {
            keira_arch::cpu::percpu::init_percpu(0);
            keira_arch::cpu::percpu::set_current_kernel_stack(stack_top as u64);
        }
        #[cfg(test)]
        {
            let _ = stack_top;
        }
    }

    #[cfg(target_arch = "x86")]
    {
        let _ = stack_top;
    }
}

/// Initializes User Mode structures: populates GDT TSS entry, reloads GDT,
/// loads TSS register and configures syscall MSR registers.
///
/// # Safety
/// Modifies processor privilege registers, MSRs and hardware task state segment configuration.
pub unsafe fn init_user_mode() {
    keira_task::scheduler::register_task_cleanup_hook(crate::dispatcher::syscall_task_cleanup_hook);

    let stack_pages = 8usize;
    let stack_frame = pmm::alloc_contiguous_frames(stack_pages).unwrap_or(0x1000_0000);
    BOOT_KERNEL_STACK_TOP = (stack_frame + (stack_pages as u64) * pmm::PAGE_SIZE) as usize;

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
