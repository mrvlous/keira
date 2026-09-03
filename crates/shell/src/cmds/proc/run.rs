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
use keira_mem::pmm;
#[cfg(target_arch = "x86_64")]
use keira_mem::vmm;

extern "C" {
    fn jump_to_user(entry: u64, stack: u64);
}

#[cfg(target_arch = "x86")]
const USER_STACK_TOP: u64 = 0x07FFF000 - 16;

#[cfg(target_arch = "x86_64")]
const USER_DEFAULT_BRK: u64 = 0x600000000000;
#[cfg(target_arch = "x86_64")]
const USER_STACK_TOP: u64 = 0x7FFFFFE00000 - 16;

const AT_NULL: u64 = 0;
const AT_PAGESZ: u64 = 6;
const AT_BASE: u64 = 7;
const AT_FLAGS: u64 = 8;
const AT_ENTRY: u64 = 9;
const AT_UID: u64 = 11;
const AT_EUID: u64 = 12;
const AT_CLKTCK: u64 = 17;
const AT_RANDOM: u64 = 25;

/// Format System V AMD64 ABI user stack with argc, argv pointers, envp pointers, auxv, and string data.
#[cfg(target_arch = "x86_64")]
unsafe fn setup_user_stack_args_64(
    page_ptr: *mut u8,
    top_page_vaddr: u64,
    args: &[&str],
    entry_point: u64,
) -> u64 {
    let mut offset = 4096usize - 16;
    let mut arg_vaddrs = [0u64; 16];
    let argc = if args.is_empty() {
        1
    } else {
        args.len().min(16)
    };

    for (i, &arg) in args[..argc].iter().enumerate() {
        let bytes = arg.as_bytes();
        let len = bytes.len() + 1;
        if offset < len + 256 {
            break;
        }
        offset -= len;
        core::ptr::copy_nonoverlapping(bytes.as_ptr(), page_ptr.add(offset), bytes.len());
        *page_ptr.add(offset + bytes.len()) = 0;
        arg_vaddrs[i] = top_page_vaddr + offset as u64;
    }

    // Allocate 16 random entropy bytes for AT_RANDOM stack canary seed
    offset = offset.saturating_sub(16);
    let random_entropy = [
        0x4bu8, 0x65, 0x69, 0x72, 0x61, 0x5f, 0x72, 0x6e, 0x67, 0x5f, 0x73, 0x65, 0x65, 0x64, 0x32,
        0x36,
    ];
    core::ptr::copy_nonoverlapping(random_entropy.as_ptr(), page_ptr.add(offset), 16);
    let random_vaddr = top_page_vaddr + offset as u64;

    // 16-byte align before pointer words
    offset &= !15;

    let auxv: [(u64, u64); 8] = [
        (AT_PAGESZ, 4096),
        (AT_ENTRY, entry_point),
        (AT_BASE, 0),
        (AT_FLAGS, 0),
        (AT_UID, 0),
        (AT_EUID, 0),
        (AT_CLKTCK, 100),
        (AT_RANDOM, random_vaddr),
    ];
    let auxv_words = (auxv.len() + 1) * 2;

    // Total words: argc(1) + argv pointers(argc) + argv_null(1) + envp_null(1) + auxv_words
    let total_words = 1 + argc + 1 + 1 + auxv_words;
    if total_words % 2 != 0 {
        offset = offset.saturating_sub(8);
    }
    offset = offset.saturating_sub(total_words * 8);

    let stack_u64 = page_ptr.add(offset) as *mut u64;
    let mut w_idx = 0;

    // 1. argc
    *stack_u64.add(w_idx) = argc as u64;
    w_idx += 1;

    // 2. argv[0..argc]
    for i in 0..argc {
        *stack_u64.add(w_idx) = arg_vaddrs[i];
        w_idx += 1;
    }

    // 3. argv NULL terminator
    *stack_u64.add(w_idx) = 0;
    w_idx += 1;

    // 4. envp NULL terminator
    *stack_u64.add(w_idx) = 0;
    w_idx += 1;

    // 5. auxv table
    for (tag, val) in auxv.iter() {
        *stack_u64.add(w_idx) = *tag;
        *stack_u64.add(w_idx + 1) = *val;
        w_idx += 2;
    }
    *stack_u64.add(w_idx) = AT_NULL;
    *stack_u64.add(w_idx + 1) = 0;

    top_page_vaddr + offset as u64
}

/// Format 32-bit user stack with argc, argv pointers, envp pointers, auxv, and string data.
#[cfg(target_arch = "x86")]
unsafe fn setup_user_stack_args_32(
    page_ptr: *mut u8,
    top_page_vaddr: u64,
    args: &[&str],
    entry_point: u64,
) -> u64 {
    let mut offset = 4096usize - 16;
    let mut arg_vaddrs = [0u32; 16];
    let argc = if args.is_empty() {
        1
    } else {
        args.len().min(16)
    };

    for (i, &arg) in args[..argc].iter().enumerate() {
        let bytes = arg.as_bytes();
        let len = bytes.len() + 1;
        if offset < len + 256 {
            break;
        }
        offset -= len;
        core::ptr::copy_nonoverlapping(bytes.as_ptr(), page_ptr.add(offset), bytes.len());
        *page_ptr.add(offset + bytes.len()) = 0;
        arg_vaddrs[i] = (top_page_vaddr + offset as u64) as u32;
    }

    // Allocate 16 random entropy bytes for AT_RANDOM stack canary seed
    offset = offset.saturating_sub(16);
    let random_entropy = [
        0x4bu8, 0x65, 0x69, 0x72, 0x61, 0x5f, 0x72, 0x6e, 0x67, 0x5f, 0x73, 0x65, 0x65, 0x64, 0x32,
        0x36,
    ];
    core::ptr::copy_nonoverlapping(random_entropy.as_ptr(), page_ptr.add(offset), 16);
    let random_vaddr = (top_page_vaddr + offset as u64) as u32;

    offset &= !15;

    let auxv: [(u32, u32); 8] = [
        (AT_PAGESZ as u32, 4096),
        (AT_ENTRY as u32, entry_point as u32),
        (AT_BASE as u32, 0),
        (AT_FLAGS as u32, 0),
        (AT_UID as u32, 0),
        (AT_EUID as u32, 0),
        (AT_CLKTCK as u32, 100),
        (AT_RANDOM as u32, random_vaddr),
    ];
    let auxv_words = (auxv.len() + 1) * 2;

    // Allocate argv vector array [argv[0], ..., argv[argc-1], NULL, envp_null(1), auxv]
    let table_words = (argc + 1) + 1 + auxv_words;
    offset = offset.saturating_sub(table_words * 4);
    let argv_table_offset = offset;
    let argv_table_ptr = page_ptr.add(argv_table_offset) as *mut u32;

    let mut w_idx = 0;
    for i in 0..argc {
        *argv_table_ptr.add(w_idx) = arg_vaddrs[i];
        w_idx += 1;
    }
    *argv_table_ptr.add(w_idx) = 0; // argv NULL terminator
    w_idx += 1;
    *argv_table_ptr.add(w_idx) = 0; // envp NULL terminator
    w_idx += 1;
    for (tag, val) in auxv.iter() {
        *argv_table_ptr.add(w_idx) = *tag;
        *argv_table_ptr.add(w_idx + 1) = *val;
        w_idx += 2;
    }
    *argv_table_ptr.add(w_idx) = AT_NULL as u32;
    *argv_table_ptr.add(w_idx + 1) = 0;

    offset &= !15;

    // Allocate cdecl call frame: [ret_dummy(1), argc(1), argv_ptr(1), envp_ptr(1)]
    offset = offset.saturating_sub(16);
    let stack_u32 = page_ptr.add(offset) as *mut u32;

    *stack_u32.add(0) = 0; // Dummy return address
    *stack_u32.add(1) = argc as u32; // argc at [ESP+4]
    *stack_u32.add(2) = (top_page_vaddr + argv_table_offset as u64) as u32; // argv at [ESP+8]
    *stack_u32.add(3) = (top_page_vaddr + (argv_table_offset + (argc + 1) * 4) as u64) as u32; // envp at [ESP+12]

    top_page_vaddr + offset as u64
}

/// Execute a freestanding user mode ELF program in an isolated address space with CLI arguments.
pub unsafe fn run_user_program(filename: &str, args: &[&str]) -> Result<(), &'static str> {
    #[cfg(target_arch = "x86")]
    {
        let prev_sched = keira_task::scheduler::SCHEDULER_INITIALIZED;
        keira_task::scheduler::SCHEDULER_INITIALIZED = false;

        let entry_point = load_elf(filename)?;
        let top_stack_page = USER_STACK_TOP & !(pmm::PAGE_SIZE - 1);
        let ptr = top_stack_page as *mut u8;
        let initial_user_rsp = setup_user_stack_args_32(ptr, top_stack_page, args, entry_point);

        let _job_id = keira_task::signal::add_job(1, filename, true);
        jump_to_user(entry_point, initial_user_rsp);
        keira_task::signal::remove_job_by_pid(1);

        keira_task::scheduler::SCHEDULER_INITIALIZED = prev_sched;
        core::arch::asm!("sti");

        return Ok(());
    }

    #[cfg(target_arch = "x86_64")]
    {
        let parent_pml4 = vmm::active_pml4();
        let child_pml4 = vmm::clone_kernel_pml4()?;

        let prev_sched = keira_task::scheduler::SCHEDULER_INITIALIZED;
        keira_task::scheduler::SCHEDULER_INITIALIZED = false;

        if let Some(ref mut task) = keira_task::scheduler::TASKS[0] {
            task.pml4_phys = child_pml4;
        }

        vmm::switch_address_space(child_pml4);

        let cleanup_and_restore = |child: u64, brk: u64| {
            if let Some(ref mut task) = keira_task::scheduler::TASKS[0] {
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
            vmm::switch_address_space(parent_pml4);
            keira_task::scheduler::SCHEDULER_INITIALIZED = prev_sched;
            vmm::free_user_pages(child, brk);
        };

        let entry_point = match load_elf(filename) {
            Ok(ep) => ep,
            Err(e) => {
                cleanup_and_restore(child_pml4, USER_DEFAULT_BRK);
                return Err(e);
            }
        };

        // Allocate initial top stack frame (4KB); further stack frames are allocated on-demand via #PF
        let top_stack_page = USER_STACK_TOP & !(pmm::PAGE_SIZE - 1);
        let stack_frame = match pmm::alloc_frame() {
            Some(f) => f,
            None => {
                cleanup_and_restore(child_pml4, USER_DEFAULT_BRK);
                return Err("Out of memory for user stack frame");
            }
        };
        if let Err(e) = vmm::map_page(
            top_stack_page,
            stack_frame,
            vmm::PAGE_USER | vmm::PAGE_WRITABLE | vmm::PAGE_PRESENT,
        ) {
            pmm::free_frame(stack_frame);
            cleanup_and_restore(child_pml4, USER_DEFAULT_BRK);
            return Err(e);
        }
        let ptr = top_stack_page as *mut u8;
        core::ptr::write_bytes(ptr, 0, pmm::PAGE_SIZE as usize);
        let initial_user_rsp = setup_user_stack_args_64(ptr, top_stack_page, args, entry_point);

        let mut brk_end = USER_DEFAULT_BRK;
        if let Some(ref mut task) = keira_task::scheduler::TASKS[0] {
            task.program_break = USER_DEFAULT_BRK;
            task.program_break_start = USER_DEFAULT_BRK;
            task.pml4_phys = child_pml4;
        }

        let _job_id = keira_task::signal::add_job(1, filename, true);
        jump_to_user(entry_point, initial_user_rsp);
        keira_task::signal::remove_job_by_pid(1);

        if let Some(ref mut task) = keira_task::scheduler::TASKS[0] {
            brk_end = task.program_break;
        }

        cleanup_and_restore(child_pml4, brk_end);

        core::arch::asm!("sti");

        Ok(())
    }
}

/// Resolve an ELF binary path and execute it with CLI arguments, returning true if found.
pub fn run_direct_with_args(arg: &str, args: &[&str]) -> bool {
    unsafe {
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
            return false;
        }

        vga::print_str("Loading ELF binary: ");
        vga::print_str(resolved_str);
        vga::print_str("\n");

        match run_user_program(resolved_str, args) {
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

        true
    }
}

/// Backward compatible run_direct helper.
pub fn run_direct(arg: &str) -> bool {
    let args = [arg];
    run_direct_with_args(arg, &args)
}

pub fn run(parts: &mut core::str::SplitWhitespace) {
    let mut args_buf: [&str; 16] = [""; 16];
    let mut arg_count = 0;

    while let Some(part) = parts.next() {
        if arg_count == 0 && (part == "-h" || part == "--help") {
            vga::print_str("Usage: run <program.elf> [arg1] [arg2] ...\n\n");
            vga::print_str("Description:\n  Load and execute a freestanding user mode ELF binary program in Ring 3 user space with CLI arguments.\n\n");
            vga::print_str("Options:\n  -h, --help    Show this help message and exit\n\n");
            vga::print_str("Examples:\n  run hello.elf\n  run /apps/bin/calc.elf\n  run kcc /data/main.c -o /apps/bin/app.elf\n");
            return;
        }
        if arg_count < 16 {
            args_buf[arg_count] = part;
            arg_count += 1;
        }
    }

    if arg_count == 0 {
        vga::print_str("Usage: run <program.elf> [arg1] [arg2] ...\n");
        return;
    }

    let prog_arg = args_buf[0];
    if !run_direct_with_args(prog_arg, &args_buf[..arg_count]) {
        vga::set_color(vga::Color::LightRed, vga::Color::Black);
        vga::print_str("Error executing program: file not found\n");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    }
}
