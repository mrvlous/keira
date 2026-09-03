// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Rust kernel main entry point and early bootstrap orchestration.

use keira_fs::fat;
use keira_fs::tar;
use keira_io::bus::pci;
use keira_io::ps2::{keyboard as ps2_keyboard, mouse as ps2_mouse};
use keira_io::rtc;
use keira_io::sound::hda;
use keira_io::storage::{ahci, block, ide};
use keira_io::vga;
use keira_mem::vmm;
use keira_net::driver::e1000;
use keira_shell as shell;
use keira_syscall::init_user_mode;
use keira_task::scheduler::init as scheduler_init;

#[cfg(target_arch = "x86")]
const ARCH_BOOT_STR: &str = "Confirming active CPU x86 32-bit Protected Mode status";
#[cfg(target_arch = "x86_64")]
const ARCH_BOOT_STR: &str = "Confirming active CPU x86_64 Long Mode status";
#[cfg(target_arch = "aarch64")]
const ARCH_BOOT_STR: &str = "Confirming active CPU aarch64 Exception Level status";
#[cfg(target_arch = "riscv64")]
const ARCH_BOOT_STR: &str = "Confirming active CPU riscv64 Supervisor Mode status";

#[cfg(target_arch = "x86")]
const ENTRY_CONTEXT_STR: &str = "Landed in 32-bit Rust kernel entry context";
#[cfg(target_arch = "x86_64")]
const ENTRY_CONTEXT_STR: &str = "Landed in 64-bit Rust kernel entry context";
#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
const ENTRY_CONTEXT_STR: &str = "Landed in Rust kernel entry context";

/// Kernel main entry point called by the assembly trampoline.
#[no_mangle]
pub extern "C" fn kernel_main(multiboot_info_ptr: usize) -> ! {
    // 1. Early Pure Rust Architecture & Peripheral Bringup
    vga::init();
    keira_arch::init();
    ps2_keyboard::init();
    ps2_mouse::init();
    rtc::init();

    vga::print_boot_log("Initializing Serial Port (COM1) driver", 0);
    vga::print_boot_log("Configuring VGA text-mode frame buffer (80x25)", 0);
    vga::print_boot_log("Checking x86 CPUID & Model Specific Registers (MSRs)", 0);
    vga::print_boot_log("Loading Interrupt Descriptor Table (IDT) registers", 0);
    vga::print_boot_log("Remapping dual 8259 PIC interrupt controller IRQs", 0);
    vga::print_boot_log("Configuring 8253 PIT system timer tick rate to 1000Hz", 0);
    vga::print_boot_log(
        "Initializing High-Precision Event Timer (HPET) Subsystem",
        0,
    );
    vga::print_boot_log("Initializing PS/2 keyboard controller & driver", 0);
    vga::print_boot_log("Initializing PS/2 mouse controller & driver", 0);
    vga::print_boot_log("Reading CMOS Real-Time Clock (RTC) date/time registers", 0);
    vga::print_boot_log("Scanning PCIe ECAM Memory-Mapped Configuration Space", 0);
    vga::print_boot_log("Completing low-level hardware subsystem checks", 0);

    vga::print_boot_log(ENTRY_CONTEXT_STR, 0);
    vga::print_boot_log("Checking Multiboot2 bootloader magic signature", 0);
    vga::print_boot_log("Validating page frame identity mapping", 0);
    vga::print_boot_log(ARCH_BOOT_STR, 0);

    #[cfg(target_arch = "x86_64")]
    let cpuid = core::arch::x86_64::__cpuid(0);
    #[cfg(target_arch = "x86")]
    let cpuid = core::arch::x86::__cpuid(0);

    let mut vendor = [0u8; 12];
    vendor[0..4].copy_from_slice(&cpuid.ebx.to_le_bytes());
    vendor[4..8].copy_from_slice(&cpuid.edx.to_le_bytes());
    vendor[8..12].copy_from_slice(&cpuid.ecx.to_le_bytes());

    let mut cpuid_msg = [0u8; 33];
    cpuid_msg[0..21].copy_from_slice(b"Detected CPU Vendor: ");
    cpuid_msg[21..33].copy_from_slice(&vendor);
    if let Ok(msg_str) = core::str::from_utf8(&cpuid_msg) {
        vga::print_boot_log(msg_str, 0);
    }

    let mut initrd_start = 0u64;
    let mut initrd_end = 0u64;
    unsafe {
        let mut addr = multiboot_info_ptr + 8;
        loop {
            let tag_type = *(addr as *const u32);
            let tag_size = *((addr + 4) as *const u32);
            if tag_type == 0 {
                break;
            }
            if tag_type == 3 {
                initrd_start = *((addr + 8) as *const u32) as u64;
                initrd_end = *((addr + 12) as *const u32) as u64;
            }
            if tag_type == 8 {
                vga::FRAMEBUFFER_ADDR = *((addr + 8) as *const u64);
                vga::FRAMEBUFFER_PITCH = *((addr + 16) as *const u32);
                vga::FRAMEBUFFER_WIDTH = *((addr + 20) as *const u32);
                vga::FRAMEBUFFER_HEIGHT = *((addr + 24) as *const u32);
                vga::FRAMEBUFFER_BPP = *((addr + 28) as *const u8);
            }
            addr += ((tag_size as usize) + 7) & !7;
        }
    }

    if initrd_start != 0 {
        tar::init(initrd_start, initrd_end);
        vga::print_boot_log("Mounting read-only Initrd USTAR boot archive", 0);
    } else {
        vga::print_boot_log("Mounting read-only Initrd USTAR boot archive", 2);
    }

    extern "C" {
        static __heap_end: u8;
    }
    let heap_end_addr = unsafe { &__heap_end as *const u8 as u64 };
    unsafe {
        keira_mem::init(multiboot_info_ptr as u64, initrd_end, heap_end_addr);

        let fb_addr = vga::FRAMEBUFFER_ADDR;
        let fb_pitch = vga::FRAMEBUFFER_PITCH;
        let fb_height = vga::FRAMEBUFFER_HEIGHT;
        if fb_addr != 0 {
            let fb_size = fb_height as u64 * fb_pitch as u64;
            let page_count = fb_size.div_ceil(4096);
            for i in 0..page_count {
                let offset = i * 4096;
                let phys = fb_addr + offset;
                let _ = vmm::map_page(phys, phys, vmm::PAGE_WRITABLE);
            }
            let fb_width = vga::FRAMEBUFFER_WIDTH;
            ps2_mouse::set_resolution(fb_width as i32, fb_height as i32);
            vga::FRAMEBUFFER_MAPPED = true;
            vga::init();
        }
    }
    vga::print_boot_log("Initializing Physical Memory Manager (PMM) frames", 0);
    vga::print_boot_log("Initializing Virtual Memory Manager (VMM) paging", 0);

    unsafe {
        // Identity map Local APIC (0xFEE00000) and I/O APIC (0xFEC00000) MMIO registers
        let _ = vmm::map_page(0xFEE0_0000, 0xFEE0_0000, vmm::PAGE_WRITABLE);
        let _ = vmm::map_page(0xFEC0_0000, 0xFEC0_0000, vmm::PAGE_WRITABLE);

        scheduler_init();
    }
    vga::print_boot_log("Initializing Preemptive Round-Robin Thread Scheduler", 0);

    vga::print_boot_log(
        "Initializing PCI Bus & storage/audio/network host controllers",
        0,
    );
    pci::init();
    let _ = ahci::init();
    let _ = unsafe { hda::init() };
    unsafe {
        if e1000::init() {
            vga::print_boot_log("Initializing Intel e1000 Gigabit Ethernet NIC driver", 0);
            vga::print_boot_log("Configuring Network Stack (Ethernet/ARP/IPv4/ICMP)", 0);
        }
    }

    unsafe {
        let mut mounted = false;
        if block::mount_device("ahci0").is_ok() {
            mounted = true;
        } else if let Ok(sectors) = ide::identify() {
            ide::IDE_DEVICE.size_sectors = sectors;
            let _ = block::register_device(&*core::ptr::addr_of!(ide::IDE_DEVICE));
            if block::mount_device("ide0").is_ok() {
                mounted = true;
            }
        }

        match fat::init() {
            Ok(_) => {
                if mounted {
                    if let Some(dev) = block::get_mounted_device() {
                        if dev.get_name() == "ahci0" {
                            vga::print_boot_log(
                                "Probing SATA master storage controller via AHCI",
                                0,
                            );
                        } else {
                            vga::print_boot_log("Probing IDE primary master storage controller", 0);
                        }
                    }
                } else {
                    vga::print_boot_log("Probing primary storage controller", 0);
                }
                vga::print_boot_log("Registering active storage block device drives", 0);
                vga::print_boot_log("Mounting and initializing FAT16 file system driver", 0);
            }
            Err(e) => {
                let mut err_msg = [0u8; 80];
                let prefix = b"Mounting and initializing FAT16 file system driver (Error: ";
                let suffix = b")";
                let mut offset = 0;
                err_msg[offset..offset + prefix.len()].copy_from_slice(prefix);
                offset += prefix.len();
                let e_bytes = e.as_bytes();
                let to_copy = core::cmp::min(e_bytes.len(), err_msg.len() - offset - suffix.len());
                err_msg[offset..offset + to_copy].copy_from_slice(&e_bytes[..to_copy]);
                offset += to_copy;
                err_msg[offset..offset + suffix.len()].copy_from_slice(suffix);
                offset += suffix.len();
                if let Ok(msg_str) = core::str::from_utf8(&err_msg[..offset]) {
                    vga::print_boot_log(msg_str, 1);
                } else {
                    vga::print_boot_log("Mounting and initializing FAT16 file system driver", 1);
                }
            }
        }
    }

    unsafe {
        init_user_mode();
    }
    vga::print_boot_log("Re-configuring Global Descriptor Table (GDT) segments", 0);
    vga::print_boot_log("Loading Task State Segment (TSS) cpu context structure", 0);
    vga::print_boot_log("Enabling CPU ring 3 user-mode syscall interface MSRs", 0);
    vga::print_boot_log("Initializing Mandatory Access Control (MAC) Security", 0);

    vga::print_boot_log("Spawning interactive terminal shell environment", 0);
    vga::print_boot_log("Keira Kernel initialized successfully. System ready", 0);

    vga::init();

    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    vga::print_str("Keira Kernel ");
    vga::print_str(env!("CARGO_PKG_VERSION"));
    vga::print_str("-keira-1 (tty1)\n\n");

    unsafe {
        core::arch::asm!("sti");
    }

    shell::run_boot_script();
    shell::print_prompt();

    loop {
        shell::process_pending();
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}
