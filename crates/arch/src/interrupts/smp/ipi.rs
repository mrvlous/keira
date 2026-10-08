// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Symmetric Multiprocessing (SMP), Inter-Processor Interrupts (IPI), and cross-core TLB shootdown.

use core::sync::atomic::{AtomicUsize, Ordering};
use keira_core::sync::{IrqSpinLock, LockRank};

use crate::cpu::invlpg;
use crate::interrupts::apic;
#[cfg(target_os = "none")]
use crate::power::acpi;

pub const MAX_CORES: usize = 16;
pub const AP_TRAMPOLINE_PHYS: usize = 0x8000;

/// Interrupt vector assigned to SMP cross-core TLB shootdown (0xFD / 253).
pub const VECTOR_TLB_SHOOTDOWN: u8 = 0xFD;

/// Interrupt vector assigned to SMP cross-core rescheduling preemption (0xFE / 254).
pub const VECTOR_RESCHEDULE: u8 = 0xFE;

static TLB_TARGET_ADDR: AtomicUsize = AtomicUsize::new(0);
static TLB_ACK_COUNTER: AtomicUsize = AtomicUsize::new(0);
static TLB_SHOOTDOWN_LOCK: IrqSpinLock = IrqSpinLock::with_rank(LockRank::None);

static TLB_SHOOTDOWN_COUNT: AtomicUsize = AtomicUsize::new(0);
static TLB_IPI_SENT_COUNT: AtomicUsize = AtomicUsize::new(0);

static RESCHED_IPI_SENT_COUNT: AtomicUsize = AtomicUsize::new(0);
static RESCHED_IPI_RCVD_COUNT: AtomicUsize = AtomicUsize::new(0);

#[cfg(target_os = "none")]
extern "C" {
    static ap_trampoline_start: u8;
    static ap_trampoline_end: u8;
    static mut ap_cr3_val: usize;
    static mut ap_stack_val: usize;
    static mut ap_entry_val: usize;
    static mut ap_core_id: usize;
    static mut ap_status_flag: usize;
    #[cfg(target_arch = "x86_64")]
    fn reload_gdt();
}

#[cfg(not(target_os = "none"))]
#[allow(non_upper_case_globals, dead_code)]
static ap_trampoline_start: u8 = 0;
#[cfg(not(target_os = "none"))]
#[allow(non_upper_case_globals, dead_code)]
static ap_trampoline_end: u8 = 0;
#[cfg(not(target_os = "none"))]
#[allow(non_upper_case_globals, dead_code)]
static mut ap_cr3_val: usize = 0;
#[cfg(not(target_os = "none"))]
#[allow(non_upper_case_globals, dead_code)]
static mut ap_stack_val: usize = 0;
#[cfg(not(target_os = "none"))]
#[allow(non_upper_case_globals, dead_code)]
static mut ap_entry_val: usize = 0;
#[cfg(not(target_os = "none"))]
#[allow(non_upper_case_globals, dead_code)]
static mut ap_core_id: usize = 0;
#[cfg(not(target_os = "none"))]
#[allow(non_upper_case_globals, dead_code)]
static mut ap_status_flag: usize = 0;

#[cfg(all(not(target_os = "none"), target_arch = "x86_64"))]
#[allow(dead_code)]
unsafe fn reload_gdt() {}

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

/// Atomic rendezvous barrier counter tracking online core boot synchronization.
pub static SMP_BOOT_BARRIER: AtomicUsize = AtomicUsize::new(1);

/// Increments the SMP rendezvous barrier indicating that a core has arrived at the boot gate.
pub fn smp_barrier_arrive() -> usize {
    SMP_BOOT_BARRIER.fetch_add(1, Ordering::SeqCst) + 1
}

/// Retrieves the current arrival count at the SMP boot barrier.
pub fn smp_barrier_count() -> usize {
    SMP_BOOT_BARRIER.load(Ordering::SeqCst)
}

/// Resets the SMP rendezvous barrier back to an initial count.
pub fn smp_barrier_reset(initial: usize) {
    SMP_BOOT_BARRIER.store(initial, Ordering::SeqCst);
}

/// Retrieve the total number of currently online CPU cores.
pub fn get_online_cores_count() -> usize {
    unsafe { SMP_CORES_COUNT }
}

/// Retrieve the metadata structure of a specific initialized CPU core.
pub fn get_core_info(index: usize) -> Option<CpuCore> {
    if index >= MAX_CORES {
        None
    } else {
        unsafe { SMP_CORES[index] }
    }
}

/// Send Inter-Processor Interrupt (IPI) to a specific target Local APIC.
pub fn send_ipi(target_apic_id: u8, vector: u8) {
    #[cfg(target_os = "none")]
    unsafe {
        apic::wait_icr_idle();
        let icr_high = (target_apic_id as u32) << 24;
        let icr_low = (1 << 14) | (vector as u32);
        apic::write_reg(apic::LAPIC_ICR_HIGH_REG, icr_high);
        apic::write_reg(apic::LAPIC_ICR_LOW_REG, icr_low);
    }
    #[cfg(not(target_os = "none"))]
    {
        let _ = (target_apic_id, vector);
    }
}

/// Send INIT Inter-Processor Interrupt (IPI) to reset an Application Processor (AP).
pub fn send_init_ipi(target_apic_id: u8) {
    #[cfg(target_os = "none")]
    unsafe {
        apic::wait_icr_idle();
        let icr_high = (target_apic_id as u32) << 24;
        let icr_low = 0x0000_4500;
        apic::write_reg(apic::LAPIC_ICR_HIGH_REG, icr_high);
        apic::write_reg(apic::LAPIC_ICR_LOW_REG, icr_low);
    }
    #[cfg(not(target_os = "none"))]
    {
        let _ = target_apic_id;
    }
}

/// Send Startup Inter-Processor Interrupt (SIPI) to begin AP execution at `vector * 0x1000`.
pub fn send_startup_ipi(target_apic_id: u8, vector: u8) {
    #[cfg(target_os = "none")]
    unsafe {
        apic::wait_icr_idle();
        let icr_high = (target_apic_id as u32) << 24;
        let icr_low = 0x0000_4600 | (vector as u32);
        apic::write_reg(apic::LAPIC_ICR_HIGH_REG, icr_high);
        apic::write_reg(apic::LAPIC_ICR_LOW_REG, icr_low);
    }
    #[cfg(not(target_os = "none"))]
    {
        let _ = (target_apic_id, vector);
    }
}

/// Broadcast an Inter-Processor Interrupt (IPI) to all other CPU cores excluding the current core.
pub fn send_ipi_all_excluding_self(vector: u8) {
    #[cfg(target_os = "none")]
    unsafe {
        apic::wait_icr_idle();
        // Destination Shorthand: 11b (bits 18..19 = 3 << 18)
        // Delivery Mode: Fixed (000b)
        // Level: Assert (1 << 14)
        // Vector: bits 0..7
        let icr_low = (3 << 18) | (1 << 14) | (vector as u32);
        apic::write_reg(apic::LAPIC_ICR_LOW_REG, icr_low);
    }
    #[cfg(not(target_os = "none"))]
    {
        let _ = vector;
    }
}

/// Flushes the complete processor TLB by reloading control register CR3.
#[inline(always)]
pub fn reload_cr3() {
    #[cfg(all(target_os = "none", target_arch = "x86_64"))]
    unsafe {
        let cr3: u64;
        core::arch::asm!("mov {}, cr3", out(reg) cr3, options(nomem, nostack, preserves_flags));
        core::arch::asm!("mov cr3, {}", in(reg) cr3, options(nomem, nostack, preserves_flags));
    }
    #[cfg(all(target_os = "none", target_arch = "x86"))]
    unsafe {
        let cr3: u32;
        core::arch::asm!("mov {:e}, cr3", out(reg) cr3, options(nomem, nostack, preserves_flags));
        core::arch::asm!("mov cr3, {:e}", in(reg) cr3, options(nomem, nostack, preserves_flags));
    }
}

/// Execute cross-core synchronous TLB Shootdown to invalidate page address across all CPU cores.
///
/// If `vaddr == 0`, flushes the entire TLB via CR3 reload on all cores.
pub fn tlb_shootdown(vaddr: u64) {
    TLB_SHOOTDOWN_COUNT.fetch_add(1, Ordering::Relaxed);
    let online_cores = unsafe { SMP_CORES_COUNT };

    if online_cores <= 1 {
        // Fast path for uniprocessor mode: local invalidation only
        if vaddr != 0 {
            invlpg(vaddr as usize);
        } else {
            reload_cr3();
        }
        return;
    }

    // SMP Synchronous Multi-Core Rendezvous
    let _guard = TLB_SHOOTDOWN_LOCK.lock();
    TLB_TARGET_ADDR.store(vaddr as usize, Ordering::Release);
    TLB_ACK_COUNTER.store(0, Ordering::Release);

    let remote_cores = online_cores.saturating_sub(1);
    if remote_cores > 0 {
        TLB_IPI_SENT_COUNT.fetch_add(remote_cores, Ordering::Relaxed);
        send_ipi_all_excluding_self(VECTOR_TLB_SHOOTDOWN);
    }

    // Invalidate local CPU core's TLB
    if vaddr != 0 {
        invlpg(vaddr as usize);
    } else {
        reload_cr3();
    }

    // Spin-wait until all remote active cores acknowledge completion
    let mut spins = 0usize;
    while TLB_ACK_COUNTER.load(Ordering::Acquire) < remote_cores {
        core::hint::spin_loop();
        spins += 1;
        if spins > 10_000_000 {
            // Failsafe exit to avoid deadlock if a remote processor halted unexpectedly
            break;
        }
    }
}

/// Handler invoked by `isr_tlb_shootdown` when Vector 0xFD is trapped.
#[no_mangle]
pub extern "C" fn tlb_shootdown_handler() {
    let target = TLB_TARGET_ADDR.load(Ordering::Acquire);
    if target != 0 {
        invlpg(target);
    } else {
        reload_cr3();
    }

    unsafe {
        apic::eoi();
    }

    TLB_ACK_COUNTER.fetch_add(1, Ordering::Release);
}

/// Retrieves TLB shootdown metrics: (total shootdowns initiated, total remote IPIs sent).
pub fn get_tlb_shootdown_stats() -> (u64, u64) {
    (
        TLB_SHOOTDOWN_COUNT.load(Ordering::Relaxed) as u64,
        TLB_IPI_SENT_COUNT.load(Ordering::Relaxed) as u64,
    )
}

/// Send Reschedule IPI (Vector 0xFE) to a specific target CPU core.
pub fn smp_send_reschedule(target_cpu_id: usize) {
    let online_cores = unsafe { SMP_CORES_COUNT };
    if online_cores <= 1 || target_cpu_id >= MAX_CORES {
        return;
    }

    if let Some(core) = get_core_info(target_cpu_id) {
        if core.status == CoreStatus::Online {
            RESCHED_IPI_SENT_COUNT.fetch_add(1, Ordering::Relaxed);
            send_ipi(core.apic_id, VECTOR_RESCHEDULE);
        }
    }
}

/// Send Reschedule IPI broadcast (Vector 0xFE) to all remote active CPU cores excluding self.
pub fn smp_send_reschedule_all_excluding_self() {
    let online_cores = unsafe { SMP_CORES_COUNT };
    if online_cores <= 1 {
        return;
    }
    let remote = online_cores.saturating_sub(1);
    if remote > 0 {
        RESCHED_IPI_SENT_COUNT.fetch_add(remote, Ordering::Relaxed);
        send_ipi_all_excluding_self(VECTOR_RESCHEDULE);
    }
}

/// Handler invoked by `isr_reschedule` when Vector 0xFE is trapped.
#[no_mangle]
pub extern "C" fn reschedule_ipi_handler() {
    RESCHED_IPI_RCVD_COUNT.fetch_add(1, Ordering::Relaxed);
    unsafe {
        apic::eoi();
    }
}

/// Retrieves Reschedule IPI metrics: (total sent, total received).
pub fn get_resched_ipi_stats() -> (u64, u64) {
    (
        RESCHED_IPI_SENT_COUNT.load(Ordering::Relaxed) as u64,
        RESCHED_IPI_RCVD_COUNT.load(Ordering::Relaxed) as u64,
    )
}

/// Application Processor (AP) kernel main entry point and idle worker loop.
///
/// # Safety
/// Invoked on a secondary CPU core after the real-mode trampoline transitions
/// the core into 64-bit Long Mode (x86_64) or 32-bit Protected Mode (i686).
#[no_mangle]
pub unsafe extern "C" fn ap_main(core_id: usize) -> ! {
    #[cfg(target_os = "none")]
    {
        // 1. Enable Local APIC on this core
        apic::enable_lapic();

        // 2. Load the global Interrupt Descriptor Table into IDTR
        crate::interrupts::idt::load_current_idt();

        #[cfg(target_arch = "x86_64")]
        {
            // 3. Reload full 64-bit kernel GDT
            reload_gdt();
        }

        // 4. Configure Per-CPU GS Base MSR for race-free swapgs syscall re-entrancy
        crate::cpu::percpu::init_percpu(core_id);

        // 5. Update core status to Online
        if core_id < MAX_CORES {
            if let Some(ref mut core) = SMP_CORES[core_id] {
                core.status = CoreStatus::Online;
            }
        }

        // 6. Signal the Bootstrap Processor (BSP) via the trampoline parameter block
        let start_addr = core::ptr::addr_of!(ap_trampoline_start) as usize;
        let offset_flag = (core::ptr::addr_of!(ap_status_flag) as usize) - start_addr;
        let param_flag = (AP_TRAMPOLINE_PHYS + offset_flag) as *mut usize;
        core::ptr::write_volatile(param_flag, 1);

        // 7. Enable hardware interrupts on this AP core
        core::arch::asm!("sti", options(nomem, nostack, preserves_flags));

        // 8. Enter low-power AP idle loop waiting for scheduler IPIs or timer ticks
        loop {
            core::hint::spin_loop();
            core::arch::asm!("hlt", options(nomem, nostack, preserves_flags));
        }
    }

    #[cfg(not(target_os = "none"))]
    {
        let _ = core_id;
        loop {
            core::hint::spin_loop();
        }
    }
}

/// Initialize SMP subsystem and discover physical/logical CPU cores via ACPI MADT (or CPUID fallback).
pub fn init_smp() {
    unsafe {
        if SMP_INITIALIZED {
            return;
        }

        let bsp_apic_id = (apic::get_current_lapic_id() & 0xFF) as u8;

        crate::cpu::percpu::init_percpu(0);

        // Register BSP (Bootstrap Processor)
        SMP_CORES[0] = Some(CpuCore {
            core_id: 0,
            apic_id: bsp_apic_id,
            is_bsp: true,
            status: CoreStatus::Online,
        });

        #[cfg(target_os = "none")]
        {
            // 1. Copy real-mode trampoline code into physical address 0x8000
            let start_addr = core::ptr::addr_of!(ap_trampoline_start) as usize;
            let end_addr = core::ptr::addr_of!(ap_trampoline_end) as usize;
            let trampoline_len = end_addr - start_addr;

            core::ptr::copy_nonoverlapping(
                start_addr as *const u8,
                AP_TRAMPOLINE_PHYS as *mut u8,
                trampoline_len,
            );

            // 2. Compute dynamic parameter offsets relative to 0x8000
            let offset_cr3 = (core::ptr::addr_of!(ap_cr3_val) as usize) - start_addr;
            let offset_stack = (core::ptr::addr_of!(ap_stack_val) as usize) - start_addr;
            let offset_entry = (core::ptr::addr_of!(ap_entry_val) as usize) - start_addr;
            let offset_core_id = (core::ptr::addr_of!(ap_core_id) as usize) - start_addr;
            let offset_flag = (core::ptr::addr_of!(ap_status_flag) as usize) - start_addr;

            let param_cr3 = (AP_TRAMPOLINE_PHYS + offset_cr3) as *mut usize;
            let param_stack = (AP_TRAMPOLINE_PHYS + offset_stack) as *mut usize;
            let param_entry = (AP_TRAMPOLINE_PHYS + offset_entry) as *mut usize;
            let param_core_id = (AP_TRAMPOLINE_PHYS + offset_core_id) as *mut usize;
            let param_flag = (AP_TRAMPOLINE_PHYS + offset_flag) as *mut usize;

            // Populate active CR3 root and Rust entrypoint function pointer
            #[cfg(target_arch = "x86_64")]
            let cr3_val = {
                let mut cr3: u64;
                core::arch::asm!("mov {}, cr3", out(reg) cr3, options(nomem, nostack, preserves_flags));
                cr3 as usize
            };
            #[cfg(target_arch = "x86")]
            let cr3_val = {
                let mut cr3: u32;
                core::arch::asm!("mov {:e}, cr3", out(reg) cr3, options(nomem, nostack, preserves_flags));
                cr3 as usize
            };

            core::ptr::write_volatile(param_cr3, cr3_val);
            core::ptr::write_volatile(param_entry, ap_main as *const () as usize);

            let acpi_topo = acpi::get_acpi_topology();
            if acpi_topo.madt_found && acpi_topo.core_count > 0 {
                // Hardware ACPI MADT topology discovery
                let mut registered = 1;
                for i in 0..acpi_topo.core_count {
                    let target_apic_id = acpi_topo.cores[i].apic_id;
                    if target_apic_id != bsp_apic_id && registered < MAX_CORES {
                        SMP_CORES[registered] = Some(CpuCore {
                            core_id: registered as u8,
                            apic_id: target_apic_id,
                            is_bsp: false,
                            status: CoreStatus::Booting,
                        });

                        let stack_top = (core::ptr::addr_of!(
                            crate::cpu::percpu::PER_CPU_STACKS[registered].stack
                        ) as usize)
                            + crate::cpu::percpu::PER_CPU_STACK_SIZE;

                        core::ptr::write_volatile(param_stack, stack_top);
                        core::ptr::write_volatile(param_core_id, registered);
                        core::ptr::write_volatile(param_flag, 0);

                        crate::cpu::percpu::init_percpu_data(registered);

                        // INIT-SIPI-SIPI sequence
                        send_init_ipi(target_apic_id);
                        crate::timers::hpet::delay_millis(10);
                        send_startup_ipi(target_apic_id, 0x08);

                        let mut online = false;
                        for _ in 0..100 {
                            crate::timers::hpet::delay_micros(100);
                            if core::ptr::read_volatile(param_flag) == 1 {
                                online = true;
                                break;
                            }
                        }

                        if !online {
                            send_startup_ipi(target_apic_id, 0x08);
                            for _ in 0..200 {
                                crate::timers::hpet::delay_micros(100);
                                if core::ptr::read_volatile(param_flag) == 1 {
                                    online = true;
                                    break;
                                }
                            }
                        }

                        if online {
                            if let Some(ref mut core) = SMP_CORES[registered] {
                                core.status = CoreStatus::Online;
                            }
                        } else if let Some(ref mut core) = SMP_CORES[registered] {
                            core.status = CoreStatus::Offline;
                        }
                        registered += 1;
                    }
                }
                SMP_CORES_COUNT = registered;
                SMP_INITIALIZED = true;
                return;
            }

            // Fallback: Query CPU topology from CPUID Leaf 1 when ACPI is not present
            #[cfg(target_arch = "x86_64")]
            let leaf1 = core::arch::x86_64::__cpuid(1);
            #[cfg(target_arch = "x86")]
            let leaf1 = core::arch::x86::__cpuid(1);

            let max_logical_cores = ((leaf1.ebx >> 16) & 0xFF) as usize;
            let detected_cores = if max_logical_cores > 0 && max_logical_cores <= MAX_CORES {
                max_logical_cores
            } else {
                1
            };

            let mut registered = 1;
            for core_id in 1..detected_cores {
                let target_apic_id = core_id as u8;
                if target_apic_id != bsp_apic_id && registered < MAX_CORES {
                    SMP_CORES[registered] = Some(CpuCore {
                        core_id: registered as u8,
                        apic_id: target_apic_id,
                        is_bsp: false,
                        status: CoreStatus::Booting,
                    });

                    let stack_top =
                        (core::ptr::addr_of!(crate::cpu::percpu::PER_CPU_STACKS[registered].stack)
                            as usize)
                            + crate::cpu::percpu::PER_CPU_STACK_SIZE;

                    core::ptr::write_volatile(param_stack, stack_top);
                    core::ptr::write_volatile(param_core_id, registered);
                    core::ptr::write_volatile(param_flag, 0);

                    crate::cpu::percpu::init_percpu_data(registered);

                    send_init_ipi(target_apic_id);
                    crate::timers::hpet::delay_millis(10);
                    send_startup_ipi(target_apic_id, 0x08);

                    let mut online = false;
                    for _ in 0..100 {
                        crate::timers::hpet::delay_micros(100);
                        if core::ptr::read_volatile(param_flag) == 1 {
                            online = true;
                            break;
                        }
                    }

                    if !online {
                        send_startup_ipi(target_apic_id, 0x08);
                        for _ in 0..200 {
                            crate::timers::hpet::delay_micros(100);
                            if core::ptr::read_volatile(param_flag) == 1 {
                                online = true;
                                break;
                            }
                        }
                    }

                    if online {
                        if let Some(ref mut core) = SMP_CORES[registered] {
                            core.status = CoreStatus::Online;
                        }
                    } else if let Some(ref mut core) = SMP_CORES[registered] {
                        core.status = CoreStatus::Offline;
                    }
                    registered += 1;
                }
            }

            SMP_CORES_COUNT = registered;
            SMP_INITIALIZED = true;
        }

        #[cfg(not(target_os = "none"))]
        {
            SMP_CORES_COUNT = 1;
            SMP_INITIALIZED = true;
        }
    }
}
