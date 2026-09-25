// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Master kernel assembly crate linking all modular subsystems into a freestanding binary.

#![cfg_attr(target_os = "none", no_std)]
#![cfg_attr(target_os = "none", no_main)]

pub mod boot;
pub mod diagnostic;
pub mod init;
pub mod runtime;

#[cfg(test)]
mod tests;

pub use keira_arch as arch;
pub use keira_core as core_subsystem;
pub use keira_crypto as crypto;
pub use keira_fs as fs;
pub use keira_io as io;
pub use keira_ipc as ipc;
pub use keira_mem as mem;
pub use keira_net as net;
pub use keira_shell as shell;
pub use keira_syscall as syscall;
pub use keira_task as task;

/// Kernel main entry point called by the assembly trampoline (`entry64.asm` / `entry.asm`).
#[no_mangle]
pub extern "C" fn kernel_main(multiboot_info_ptr: usize) -> ! {
    // Stage 1: Early peripheral & architecture bringup
    boot::early_bringup();

    // Stage 2: CPU architecture detection & CPUID confirmation
    boot::detect_cpu();

    // Stage 3: Parse Multiboot2 bootloader payload tags
    let payload = unsafe { boot::parse_multiboot2(multiboot_info_ptr) };

    // Stage 4: Memory management, paging, heap & framebuffer
    unsafe {
        init::init_memory(multiboot_info_ptr, payload.initrd_end);
    }

    // Stage 5: ACPI topology, HPET timer MMIO & SMP cores
    unsafe {
        init::init_smp_and_timers(payload.acpi_rsdp_ptr);
    }

    // Stage 6: TPM 2.0 security enclave & measured boot
    unsafe {
        init::init_security_and_measure(payload.initrd_start, payload.initrd_end);
    }

    // Stage 7: Storage controller probe, FAT16 root mount & VMM hooks
    unsafe {
        init::init_storage_and_fs();
    }

    // Stage 8: Network interface card & stack
    unsafe {
        init::init_network();
    }

    // Stage 9: Ring 3 userspace context transition
    unsafe {
        runtime::enter_userspace();
    }

    // Stage 10: Interactive shell & main kernel event loop
    runtime::enter_main_loop();
}
