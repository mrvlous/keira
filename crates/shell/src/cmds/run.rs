// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

#![allow(unused_variables, unused_unsafe)]

//! Implementation of the 'run' shell command to launch Ring 3 user space ELF programs.

use keira_fs::elf::loader::load_elf;
use keira_io::vga;
use keira_mem::{pmm, vmm};

extern "C" {
    fn jump_to_user(entry: u64, stack: u64);
}

/// Execute a freestanding user mode ELF program in an isolated address space.
pub unsafe fn run_user_program(filename: &str) -> Result<(), &'static str> {
    let child_pml4 = vmm::clone_kernel_pml4()?;
    let parent_pml4 = vmm::active_pml4();

    vmm::switch_address_space(child_pml4);

    let entry_point = match load_elf(filename) {
        Ok(ep) => ep,
        Err(e) => {
            vmm::switch_address_space(parent_pml4);
            vmm::free_user_pages(child_pml4, 0x600000000000);
            return Err(e);
        }
    };

    let stack_pages = 16;
    let stack_bottom_vaddr: u64 = 0x7FFFFFE00000;
    for p in 0..stack_pages {
        let page_vaddr = stack_bottom_vaddr + (p * pmm::PAGE_SIZE);
        let stack_frame = match pmm::alloc_frame() {
            Some(f) => f,
            None => {
                vmm::switch_address_space(parent_pml4);
                vmm::free_user_pages(child_pml4, 0x600000000000);
                return Err("Out of memory for user stack frame");
            }
        };
        vmm::map_page(
            page_vaddr,
            stack_frame,
            vmm::PAGE_USER | vmm::PAGE_WRITABLE | vmm::PAGE_PRESENT,
        )?;
        let ptr = page_vaddr as *mut u8;
        core::ptr::write_bytes(ptr, 0, pmm::PAGE_SIZE as usize);
    }
    let user_stack_top_vaddr: u64 = stack_bottom_vaddr + (stack_pages * pmm::PAGE_SIZE);

    if let Some(ref mut task) = keira_task::scheduler::TASKS[0] {
        task.program_break = 0x600000000000;
        task.program_break_start = 0x600000000000;
        task.fds = [keira_task::types::FileDescriptor::new(); 8];
        task.pml4_phys = child_pml4;
    }

    let prev_sched = keira_task::scheduler::SCHEDULER_INITIALIZED;
    keira_task::scheduler::SCHEDULER_INITIALIZED = false;

    jump_to_user(entry_point, user_stack_top_vaddr - 16);

    vmm::switch_address_space(parent_pml4);

    let mut final_brk = 0x600000000000;
    if let Some(ref mut task) = keira_task::scheduler::TASKS[0] {
        final_brk = task.program_break;
        task.pml4_phys = parent_pml4;
        for fd in 0..8 {
            if task.fds[fd].is_open {
                if task.fds[fd].write_mode {
                    if let Ok(path_str) =
                        core::str::from_utf8(&task.fds[fd].path[..task.fds[fd].path_len])
                    {
                        keira_fs::lock::release_lock(path_str, 0);
                    }
                }
                task.fds[fd].is_open = false;
            }
        }
    }

    vmm::free_user_pages(child_pml4, final_brk);

    keira_task::scheduler::SCHEDULER_INITIALIZED = prev_sched;
    core::arch::asm!("sti");

    Ok(())
}

pub fn run(parts: &mut core::str::SplitWhitespace) {
    unsafe {
        let arg = match parts.next() {
            Some("-h") | Some("--help") => {
                vga::print_str("Usage: run <program.elf>\n\n");
                vga::print_str("Description:\n  Load and execute a freestanding user mode x86_64 ELF binary program in Ring 3 user space.\n\n");
                vga::print_str("Options:\n  -h, --help    Show this help message and exit\n\n");
                vga::print_str("Examples:\n  run hello.elf\n  run kcc\n");
                return;
            }
            Some(s) => s,
            None => {
                vga::print_str("Usage: run <program.elf>\n");
                return;
            }
        };

        let mut path_buf = [0u8; 128];
        let mut resolved_str = "";
        let mut found = false;

        let mut write_path = |pref: &str, name: &str, suff: &str| -> Option<&'static str> {
            let pref_bytes = pref.as_bytes();
            let name_bytes = name.as_bytes();
            let suff_bytes = suff.as_bytes();
            let total_len = pref_bytes.len() + name_bytes.len() + suff_bytes.len();
            if total_len > 127 {
                return None;
            }
            let ptr = &mut path_buf[0] as *mut u8;
            core::ptr::copy_nonoverlapping(pref_bytes.as_ptr(), ptr, pref_bytes.len());
            core::ptr::copy_nonoverlapping(
                name_bytes.as_ptr(),
                ptr.add(pref_bytes.len()),
                name_bytes.len(),
            );
            core::ptr::copy_nonoverlapping(
                suff_bytes.as_ptr(),
                ptr.add(pref_bytes.len() + name_bytes.len()),
                suff_bytes.len(),
            );

            core::str::from_utf8(core::slice::from_raw_parts(ptr, total_len)).ok()
        };

        if keira_fs::vfs::exists(arg) {
            resolved_str = arg;
            found = true;
        }

        if !found && !arg.ends_with(".elf") {
            if let Some(p) = write_path("", arg, ".elf") {
                if keira_fs::vfs::exists(p) {
                    resolved_str = p;
                    found = true;
                }
            }
        }

        let prefixes = ["/apps/bin/", "/initrd/apps/bin/", "/"];
        let suffixes = ["", ".elf"];

        if !found {
            'outer: for &pref in &prefixes {
                for &suff in &suffixes {
                    if let Some(p) = write_path(pref, arg, suff) {
                        if keira_fs::vfs::exists(p) {
                            resolved_str = p;
                            found = true;
                            break 'outer;
                        }
                    }
                }
            }
        }

        if !found {
            vga::set_color(vga::Color::LightRed, vga::Color::Black);
            vga::print_str("Error executing program: file not found\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            return;
        }

        vga::print_str("Loading ELF binary: ");
        vga::print_str(resolved_str);
        vga::print_str("\n");

        match run_user_program(resolved_str) {
            Ok(_) => {
                vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                vga::print_str("Program exited normally.\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            }
            Err(e) => {
                vga::set_color(vga::Color::LightRed, vga::Color::Black);
                vga::print_str("Error executing program: ");
                vga::print_str(e);
                vga::print_str("\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            }
        }
    }
}
