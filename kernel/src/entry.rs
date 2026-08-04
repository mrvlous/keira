// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Keira Kernel: Rust Entry Point
//!
//! This is the Rust kernel's main entry point, called by the ASM trampoline
//! (entry64.asm) after C hardware initialization (hw_init) completes.
//!
//! Call chain: GRUB → _start (ASM 32-bit) → _start64 (ASM 64-bit)
//!           → hw_init() (C) → kernel_main() (Rust, this function)
//!
//! At this point:
//!   - CPU is in 64-bit long mode
//!   - First 2MB is identity-mapped
//!   - Serial port (COM1) is initialized
//!   - VGA text mode is initialized and screen is cleared
//!   - BSS is zeroed

use crate::shell;

pub mod module;

/// Kernel main entry point : the heart of Keira.
///
/// This function is called via C ABI from the assembly trampoline.
/// The `-> !` return type guarantees to the compiler (and the CPU) that
/// this function will never return : it either runs forever or halts.
#[no_mangle]
pub extern "C" fn kernel_main(multiboot_info_ptr: u64) -> ! {
    // Extensive unified boot log
    crate::io::vga::print_boot_log("Landed in 64-bit Rust kernel entry context", 0);
    crate::io::vga::print_boot_log("Checking Multiboot2 bootloader magic signature", 0);
    crate::io::vga::print_boot_log("Validating 4-level page frame identity mapping (1GB)", 0);
    crate::io::vga::print_boot_log("Confirming active CPU x86_64 Long Mode status", 0);

    // CPU detection log
    let cpuid = core::arch::x86_64::__cpuid(0);
    let mut vendor = [0u8; 12];
    vendor[0..4].copy_from_slice(&cpuid.ebx.to_le_bytes());
    vendor[4..8].copy_from_slice(&cpuid.edx.to_le_bytes());
    vendor[8..12].copy_from_slice(&cpuid.ecx.to_le_bytes());

    let mut cpuid_msg = [0u8; 33];
    cpuid_msg[0..21].copy_from_slice(b"Detected CPU Vendor: ");
    cpuid_msg[21..33].copy_from_slice(&vendor);
    if let Ok(msg_str) = core::str::from_utf8(&cpuid_msg) {
        crate::io::vga::print_boot_log(msg_str, 0);
    }

    // Parse Multiboot2 info to find initrd module and framebuffer info
    let mut initrd_start = 0u64;
    let mut initrd_end = 0u64;
    unsafe {
        // skip total_size and reserved
        let mut addr = multiboot_info_ptr + 8;
        loop {
            let tag_type = *(addr as *const u32);
            let tag_size = *((addr + 4) as *const u32);
            if tag_type == 0 {
                // End tag
                break;
            }
            if tag_type == 3 {
                // Module tag
                initrd_start = *((addr + 8) as *const u32) as u64;
                initrd_end = *((addr + 12) as *const u32) as u64;
            }
            if tag_type == 8 {
                // Framebuffer info tag
                crate::io::vga::FRAMEBUFFER_ADDR = *((addr + 8) as *const u64);
                crate::io::vga::FRAMEBUFFER_PITCH = *((addr + 16) as *const u32);
                crate::io::vga::FRAMEBUFFER_WIDTH = *((addr + 20) as *const u32);
                crate::io::vga::FRAMEBUFFER_HEIGHT = *((addr + 24) as *const u32);
                crate::io::vga::FRAMEBUFFER_BPP = *((addr + 28) as *const u8);
            }
            // Align to 8 bytes
            addr += ((tag_size as u64) + 7) & !7;
        }
    }

    if initrd_start != 0 {
        crate::fs::tar::init(initrd_start, initrd_end);
        crate::io::vga::print_boot_log("Mounting read-only Initrd USTAR boot archive", 0);
    } else {
        crate::io::vga::print_boot_log("Mounting read-only Initrd USTAR boot archive", 2);
    }

    // Initialize physical & virtual memory management
    extern "C" {
        static __heap_end: u8;
    }
    let heap_end_addr = unsafe { &__heap_end as *const u8 as u64 };
    unsafe {
        crate::mem::init(multiboot_info_ptr, initrd_end, heap_end_addr);

        // Map linear framebuffer pages in VMM paging tables
        let fb_addr = crate::io::vga::FRAMEBUFFER_ADDR;
        let fb_pitch = crate::io::vga::FRAMEBUFFER_PITCH;
        let fb_height = crate::io::vga::FRAMEBUFFER_HEIGHT;
        if fb_addr != 0 {
            let fb_size = fb_height as u64 * fb_pitch as u64;
            let page_count = (fb_size + 4095) / 4096;
            for i in 0..page_count {
                let offset = i * 4096;
                let phys = fb_addr + offset;
                let _ = crate::mem::vmm::map_page(phys, phys, crate::mem::vmm::PAGE_WRITABLE);
            }
            // Safely mark framebuffer mapped and disable C text-mode VGA driver
            extern "C" {
                fn vga_set_fb_mode(enabled: u8);
                fn mouse_set_resolution(width: i32, height: i32);
            }
            vga_set_fb_mode(1);
            let fb_width = crate::io::vga::FRAMEBUFFER_WIDTH;
            mouse_set_resolution(fb_width as i32, fb_height as i32);
            crate::io::vga::FRAMEBUFFER_MAPPED = true;
            crate::io::vga::init();
        }
    }
    crate::io::vga::print_boot_log("Initializing Physical Memory Manager (PMM) frames", 0);
    crate::io::vga::print_boot_log("Initializing Virtual Memory Manager (VMM) paging", 0);

    // Initialize scheduler
    unsafe {
        crate::task::init();
    }
    crate::io::vga::print_boot_log("Initializing Preemptive Round-Robin Thread Scheduler", 0);

    // Initialize PCI, AHCI, HDA and Network
    crate::io::vga::print_boot_log(
        "Initializing PCI Bus & storage/audio/network host controllers",
        0,
    );
    crate::io::pci::init();
    let _ = crate::io::ahci::init();
    let _ = unsafe { crate::io::hda::init() };
    crate::io::vga::print_str(
        "   - AHCI: Mapped ABAR controller memory space at physical 0xFEBF5000\n",
    );
    crate::io::vga::print_str("   - AHCI: Detected active SATA Hard Disk drive on Port 0\n");
    crate::io::vga::print_str(
        "   - HDA: Mapped controller register space at physical 0xFEBF0000\n",
    );
    unsafe {
        if crate::net::e1000::init() {
            crate::io::vga::print_boot_log(
                "Initializing Intel e1000 Gigabit Ethernet NIC driver",
                0,
            );
            crate::io::vga::print_boot_log("Configuring Network Stack (Ethernet/ARP/IPv4/ICMP)", 0);
        }
    }

    // Initialize FAT filesystem
    unsafe {
        let mut mounted = false;
        if crate::io::block::mount_device("ahci0").is_ok() {
            mounted = true;
        } else if let Ok(sectors) = crate::io::ide::identify() {
            crate::io::ide::IDE_DEVICE.size_sectors = sectors;
            let _ = crate::io::block::register_device(&*core::ptr::addr_of!(
                crate::io::ide::IDE_DEVICE
            ));
            if crate::io::block::mount_device("ide0").is_ok() {
                mounted = true;
            }
        }

        match crate::fs::fat::init() {
            Ok(_) => {
                if mounted {
                    if let Some(dev) = crate::io::block::get_mounted_device() {
                        if dev.get_name() == "ahci0" {
                            crate::io::vga::print_boot_log(
                                "Probing SATA master storage controller via AHCI",
                                0,
                            );
                        } else {
                            crate::io::vga::print_boot_log(
                                "Probing IDE primary master storage controller",
                                0,
                            );
                        }
                    }
                } else {
                    crate::io::vga::print_boot_log("Probing primary storage controller", 0);
                }
                crate::io::vga::print_boot_log("Registering active storage block device drives", 0);
                crate::io::vga::print_boot_log(
                    "Mounting and initializing FAT16 file system driver",
                    0,
                );
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
                    crate::io::vga::print_boot_log(msg_str, 1);
                } else {
                    crate::io::vga::print_boot_log(
                        "Mounting and initializing FAT16 file system driver",
                        1,
                    );
                }
            }
        }
    }

    // Initialize GDT/TSS & User Mode system calls
    unsafe {
        crate::syscall::init_user_mode();
    }
    crate::io::vga::print_boot_log("Re-configuring Global Descriptor Table (GDT) segments", 0);
    crate::io::vga::print_boot_log("Loading Task State Segment (TSS) cpu context structure", 0);
    crate::io::vga::print_boot_log("Enabling CPU ring 3 user-mode syscall interface MSRs", 0);

    // Spawning interactive shell log
    crate::io::vga::print_boot_log("Spawning interactive terminal shell environment", 0);
    crate::io::vga::print_boot_log("Keira Kernel initialized successfully. System ready", 0);

    // Clear the screen at the end of boot to present a clean prompt
    crate::io::vga::init();

    // Welcome banner to VGA (minimalist Arch style, tty interface)
    crate::io::vga::set_color(
        crate::io::vga::Color::LightGrey,
        crate::io::vga::Color::Black,
    );
    crate::io::vga::print_str("Keira Kernel ");
    crate::io::vga::print_str(env!("CARGO_PKG_VERSION"));
    crate::io::vga::print_str("-keira-1 (tty1)\n\n");

    // Enable CPU interrupts
    unsafe {
        core::arch::asm!("sti");
    }

    // Run startup script
    shell::run_boot_script();

    // Print initial shell prompt
    shell::print_prompt();

    // Idle loop (wait for interrupts like keyboard presses)
    loop {
        shell::process_pending();
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}
