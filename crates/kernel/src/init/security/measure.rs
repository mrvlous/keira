// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! TPM 2.0 hardware security enclave, measured boot, and integrity verification.

use keira_io::vga;
use keira_mem::vmm;
use keira_task::scheduler::init as scheduler_init;

#[cfg(target_os = "none")]
extern "C" {
    static __text_start: u8;
    static __text_end: u8;
}

#[cfg(not(target_os = "none"))]
#[allow(non_upper_case_globals)]
static __text_start: u8 = 0;
#[cfg(not(target_os = "none"))]
#[allow(non_upper_case_globals)]
static __text_end: u8 = 0;

/// Initialize TPM 2.0 controller, measure kernel and initrd hashes, and start scheduler.
///
/// # Safety
/// Caller guarantees virtual memory management and page tables are active.
pub unsafe fn init_security_and_measure(initrd_start: u64, initrd_end: u64) {
    // Identity map TPM 2.0 TIS MMIO interface (0xFED40000)
    if vmm::map_page(0xFED4_0000, 0xFED4_0000, vmm::PAGE_WRITABLE).is_ok() {
        keira_crypto::tpm::TPM_MMIO_MAPPED = true;
    }

    // Initialize TPM 2.0 security controller and baseline PCRs
    keira_crypto::tpm::init();

    // Measure genuine kernel .text segment in physical RAM (PCR 4)
    let text_start_addr = core::ptr::addr_of!(__text_start) as usize;
    let text_end_addr = core::ptr::addr_of!(__text_end) as usize;
    if text_end_addr > text_start_addr {
        let text_len = text_end_addr - text_start_addr;
        let text_slice = core::slice::from_raw_parts(text_start_addr as *const u8, text_len);
        let _ = keira_crypto::tpm::measure_kernel_code(text_slice);
    }

    // Measure Multiboot initrd payload if loaded (PCR 5)
    if initrd_start != 0 && initrd_end > initrd_start {
        let initrd_len = (initrd_end - initrd_start) as usize;
        let initrd_slice = core::slice::from_raw_parts(initrd_start as *const u8, initrd_len);
        let _ = keira_crypto::tpm::measure_initrd(initrd_slice);
    }

    scheduler_init();

    vga::print_boot_log("Initializing TPM 2.0 Hardware Security Enclave & PCRs", 0);
    vga::print_boot_log("Performing Measured Boot: Kernel Image & Initrd Archive", 0);
    vga::print_boot_log("Initializing Preemptive Round-Robin Thread Scheduler", 0);
}
