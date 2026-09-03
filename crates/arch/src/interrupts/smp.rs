// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Symmetric Multiprocessing (SMP), Inter-Processor Interrupts (IPI), and cross-core TLB shootdown.

use crate::cpu::invlpg;
use crate::interrupts::apic;

pub const MAX_CORES: usize = 16;

/// Operational status of a physical or logical CPU core.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CoreStatus {
    Offline,
    Booting,
    Online,
}

/// Metadata describing an initialized CPU core.
#[derive(Copy, Clone, Debug)]
pub struct CpuCore {
    pub core_id: u8,
    pub apic_id: u8,
    pub is_bsp: bool,
    pub status: CoreStatus,
}

pub static mut SMP_CORES: [Option<CpuCore>; MAX_CORES] = [None; MAX_CORES];
pub static mut SMP_CORES_COUNT: usize = 1;
pub static mut SMP_INITIALIZED: bool = false;

/// Send an Inter-Processor Interrupt (IPI) to a specific target APIC CPU core.
pub fn send_ipi(dest_apic_id: u8, vector: u8) {
    unsafe {
        // ICR High: Destination Field (bits 24..31)
        apic::write_reg(apic::LAPIC_ICR_HIGH_REG, (dest_apic_id as u32) << 24);
        // ICR Low: Delivery Mode Fixed (0), Edge Triggered (0), Vector (bits 0..7)
        apic::write_reg(apic::LAPIC_ICR_LOW_REG, vector as u32);
    }
}

/// Send an INIT IPI to target APIC core for hardware initialization.
pub fn send_init_ipi(dest_apic_id: u8) {
    unsafe {
        // ICR High: Destination Field
        apic::write_reg(apic::LAPIC_ICR_HIGH_REG, (dest_apic_id as u32) << 24);
        // ICR Low: Delivery Mode = 5 (INIT), Assert = 1 (Level=1), Trigger = Edge (0)
        apic::write_reg(apic::LAPIC_ICR_LOW_REG, 0x0000_4500);

        let mut timeout = 10_000;
        while (apic::read_reg(apic::LAPIC_ICR_LOW_REG) & (1 << 12)) != 0 && timeout > 0 {
            core::hint::spin_loop();
            timeout -= 1;
        }

        // De-assert INIT
        apic::write_reg(apic::LAPIC_ICR_HIGH_REG, (dest_apic_id as u32) << 24);
        apic::write_reg(apic::LAPIC_ICR_LOW_REG, 0x0000_0500);
    }
}

/// Send a Startup IPI (SIPI) with the real-mode trampoline page address.
pub fn send_startup_ipi(dest_apic_id: u8, vector_page: u8) {
    unsafe {
        apic::write_reg(apic::LAPIC_ICR_HIGH_REG, (dest_apic_id as u32) << 24);
        // Delivery Mode = 6 (Startup), Vector = vector_page
        apic::write_reg(apic::LAPIC_ICR_LOW_REG, 0x0000_0600 | (vector_page as u32));

        let mut timeout = 10_000;
        while (apic::read_reg(apic::LAPIC_ICR_LOW_REG) & (1 << 12)) != 0 && timeout > 0 {
            core::hint::spin_loop();
            timeout -= 1;
        }
    }
}

/// Execute cross-core TLB Shootdown to invalidate page address across all CPU cores.
pub fn tlb_shootdown(vaddr: u64) {
    invlpg(vaddr as usize);
}

/// Initialize SMP subsystem and discover physical/logical CPU cores via CPUID.
pub fn init_smp() {
    unsafe {
        if SMP_INITIALIZED {
            return;
        }

        let bsp_apic_id = (apic::get_current_lapic_id() & 0xFF) as u8;

        // Register BSP (Bootstrap Processor)
        SMP_CORES[0] = Some(CpuCore {
            core_id: 0,
            apic_id: bsp_apic_id,
            is_bsp: true,
            status: CoreStatus::Online,
        });

        // Query CPU topology from CPUID Leaf 1
        #[cfg(target_arch = "x86_64")]
        let leaf1 = core::arch::x86_64::__cpuid(1);
        #[cfg(target_arch = "x86")]
        let leaf1 = core::arch::x86::__cpuid(1);
        #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
        let leaf1 = core::arch::x86_64::CpuidResult {
            eax: 0,
            ebx: 0,
            ecx: 0,
            edx: 0,
        };

        let max_logical_cores = ((leaf1.ebx >> 16) & 0xFF) as usize;
        let detected_cores = if max_logical_cores > 0 && max_logical_cores <= MAX_CORES {
            max_logical_cores
        } else {
            1
        };

        // Bootstrap detected secondary AP cores via INIT-SIPI-SIPI sequence
        for core_id in 1..detected_cores {
            let target_apic_id = core_id as u8;
            if target_apic_id != bsp_apic_id {
                send_init_ipi(target_apic_id);
                send_startup_ipi(target_apic_id, 0x08);
                send_startup_ipi(target_apic_id, 0x08);

                SMP_CORES[core_id] = Some(CpuCore {
                    core_id: core_id as u8,
                    apic_id: target_apic_id,
                    is_bsp: false,
                    status: CoreStatus::Online,
                });
            }
        }

        SMP_CORES_COUNT = detected_cores;
        SMP_INITIALIZED = true;
    }
}

/// Retrieve the number of active CPU cores.
pub fn get_online_cores_count() -> usize {
    unsafe {
        if !SMP_INITIALIZED {
            init_smp();
        }
        SMP_CORES_COUNT
    }
}

/// Retrieve information about a specific core.
pub fn get_core_info(idx: usize) -> Option<CpuCore> {
    unsafe {
        if !SMP_INITIALIZED {
            init_smp();
        }
        if idx < MAX_CORES {
            SMP_CORES[idx]
        } else {
            None
        }
    }
}
