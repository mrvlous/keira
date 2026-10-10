// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Implementation of the 'run' shell command to launch Ring 3 user space ELF programs.

use keira_fs::elf::loader::load_elf;
use keira_io::vga;
use keira_mem::pmm;
#[cfg(target_arch = "x86_64")]
use keira_mem::vmm;

#[cfg(not(test))]
extern "C" {
    fn jump_to_user(entry: u64, stack: u64);
}

#[cfg(test)]
unsafe fn jump_to_user(_entry: u64, _stack: u64) {}

#[cfg(target_arch = "x86")]
const USER_DEFAULT_BRK: u64 = 0x0200_0000;
#[cfg(target_arch = "x86")]
const USER_STACK_TOP: u64 = 0x07FFF000 - 16;

#[cfg(target_arch = "x86_64")]
const USER_DEFAULT_BRK: u64 = 0x600000000000;
#[cfg(target_arch = "x86_64")]
const USER_STACK_TOP: u64 = 0x7FFFFFE00000 - 16;

use keira_task::stack::*;

/// Execute a freestanding user mode ELF program in an isolated address space with CLI arguments.
///
/// # Safety
/// Loads binary into userland pages, alters task break, sets up stack and drops privilege to Ring 3.
pub unsafe fn run_user_program(filename: &str, args: &[&str]) -> Result<(), &'static str> {
    let mut shebang_hdr = [0u8; 128];
    if let Ok(hdr_len) = keira_fs::vfs::read_file_offset(filename, 0, &mut shebang_hdr) {
        if hdr_len >= 2 && shebang_hdr[0] == b'#' && shebang_hdr[1] == b'!' {
            if let Some((interpreter, opt_arg)) =
                keira_fs::elf::parse_shebang(&shebang_hdr[..hdr_len])
            {
                let resolved_interp = if keira_fs::vfs::exists(interpreter) {
                    interpreter
                } else if interpreter == "/bin/sh" && keira_fs::vfs::exists("/bin/sh.elf") {
                    "/bin/sh.elf"
                } else {
                    interpreter
                };

                let mut new_args: [&str; 16] = [""; 16];
                let mut count = 0;
                new_args[count] = resolved_interp;
                count += 1;
                if let Some(arg) = opt_arg {
                    new_args[count] = arg;
                    count += 1;
                }
                new_args[count] = filename;
                count += 1;
                for &arg in &args[1..] {
                    if count < 16 && !arg.is_empty() {
                        new_args[count] = arg;
                        count += 1;
                    }
                }

                return run_user_program(resolved_interp, &new_args[..count]);
            }
        }
    }

    #[cfg(target_arch = "x86")]
    {
        let prev_sched = keira_task::scheduler::SCHEDULER_INITIALIZED;
        keira_task::scheduler::SCHEDULER_INITIALIZED = false;

        if let Some(ref mut task) = keira_task::scheduler::TASKS[0] {
            task.program_break = USER_DEFAULT_BRK;
            task.program_break_start = USER_DEFAULT_BRK;
        }

        let entry_point = load_elf(filename)?;
        let elf_bytes = keira_fs::elf::last_loaded_elf_slice();
        let _ = keira_crypto::tpm::measure_binary(elf_bytes, filename);
        let top_stack_page = USER_STACK_TOP & !(pmm::PAGE_SIZE - 1);
        let ptr = top_stack_page as *mut u8;
        let initial_user_rsp = setup_user_stack_32(ptr, top_stack_page, args, entry_point);

        keira_io::tty::flush_tty();
        keira_io::ps2::flush_input_queue();

        let _job_id = keira_task::signal::add_job(1, filename, true);
        jump_to_user(entry_point, initial_user_rsp);
        keira_task::signal::remove_job_by_pid(1);
        keira_task::signal::reset_signal_handlers(0);
        keira_task::security::seccomp_reset();

        keira_task::scheduler::SCHEDULER_INITIALIZED = prev_sched;
        core::arch::asm!("sti");

        return Ok(());
    }

    #[cfg(target_arch = "x86_64")]
    {
        let parent_pml4 = vmm::active_pml4();
        let child_pml4 = vmm::clone_kernel_pml4()?;

        if let Some(ref mut task) = keira_task::scheduler::TASKS[0] {
            task.pml4_phys = child_pml4;
            task.state = keira_task::TaskState::Running;
        }

        vmm::switch_address_space(child_pml4);

        let cleanup_and_restore = |child: u64, brk: u64| {
            if let Some(ref mut task) = keira_task::scheduler::TASKS[0] {
                task.pml4_phys = parent_pml4;
                task.state = keira_task::TaskState::Running;
                task.saved_sigcontext = None;
                task.signal_mask = 0;
                task.pending_signals = 0;
                for fd in 0..keira_task::MAX_FDS {
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
            vmm::free_user_pages(child, brk);
            keira_task::signal::reset_signal_handlers(0);
            keira_task::security::seccomp_reset();
        };

        let entry_point = match load_elf(filename) {
            Ok(ep) => {
                let elf_bytes = keira_fs::elf::last_loaded_elf_slice();
                let _ = keira_crypto::tpm::measure_binary(elf_bytes, filename);
                ep
            }
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
        let initial_user_rsp = setup_user_stack_64(ptr, top_stack_page, args, entry_point);

        let mut brk_end = USER_DEFAULT_BRK;
        if let Some(ref mut task) = keira_task::scheduler::TASKS[0] {
            task.program_break = USER_DEFAULT_BRK;
            task.program_break_start = USER_DEFAULT_BRK;
            task.pml4_phys = child_pml4;
        }

        keira_io::tty::flush_tty();
        keira_io::ps2::flush_input_queue();

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

/// Backward compatible run_direct helper.
pub fn run_direct(arg: &str) -> bool {
    let args = [arg];
    run_direct_with_args_mode(arg, &args, false)
}

/// Execute a binary directly with argument tokens passed from the shell command line.
pub fn run_direct_with_parts(command: &str, parts: &mut core::str::SplitWhitespace) -> bool {
    let mut args_buf: [&str; 16] = [""; 16];
    args_buf[0] = command;
    let mut arg_count = 1;

    for part in parts {
        if arg_count < 16 {
            args_buf[arg_count] = part;
            arg_count += 1;
        }
    }

    run_direct_with_args_mode(command, &args_buf[..arg_count], false)
}

/// Backward compatible run_direct_with_args helper (defaults to quiet mode).
pub fn run_direct_with_args(arg: &str, args: &[&str]) -> bool {
    run_direct_with_args_mode(arg, args, false)
}

/// Resolve an ELF binary path and execute it with CLI arguments and configurable verbosity.
pub fn run_direct_with_args_mode(arg: &str, args: &[&str], verbose: bool) -> bool {
    run_direct_with_args_inner(arg, args, verbose)
}

fn run_direct_with_args_inner(arg: &str, args: &[&str], verbose: bool) -> bool {
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

        if !found && !arg.ends_with(".elf") && !arg.ends_with(".sh") {
            if let Some(p) = write_path("", arg, ".elf") {
                if keira_fs::vfs::exists(p) {
                    resolved_str = p;
                    found = true;
                }
            }
            if !found {
                if let Some(p) = write_path("", arg, ".sh") {
                    if keira_fs::vfs::exists(p) {
                        resolved_str = p;
                        found = true;
                    }
                }
            }
        }

        let prefixes = ["/bin/", "/"];
        let suffixes = ["", ".elf", ".sh"];

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

        let is_clear_cmd = arg == "clear"
            || resolved_str.ends_with("/clear")
            || resolved_str.ends_with("/clear.elf");

        if verbose && !is_clear_cmd {
            if resolved_str.ends_with(".sh") {
                vga::print_str("Executing script: ");
            } else {
                vga::print_str("Loading ELF binary: ");
            }
            vga::print_str(resolved_str);
            vga::print_str("\n");
        }

        match run_user_program(resolved_str, args) {
            Ok(_) => {
                if verbose && !is_clear_cmd {
                    vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                    vga::print_str("Program exited normally.\n");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                }
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

pub fn run(parts: &mut core::str::SplitWhitespace) {
    let mut args_buf: [&str; 16] = [""; 16];
    let mut arg_count = 0;

    for part in parts.by_ref() {
        if arg_count == 0 && (part == "-h" || part == "--help") {
            vga::print_str("Usage: run <program.elf> [arg1] [arg2] ...\n\n");
            vga::print_str("Description:\n  Load and execute a freestanding user mode ELF binary program in Ring 3 user space with CLI arguments.\n\n");
            vga::print_str("Options:\n  -h, --help    Show this help message and exit\n\n");
            vga::print_str("Examples:\n  run hello.elf\n  run /bin/calc.elf\n  run kcc /tmp/main.c -o /bin/app.elf\n");
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
    if !run_direct_with_args_mode(prog_arg, &args_buf[..arg_count], true) {
        vga::set_color(vga::Color::LightRed, vga::Color::Black);
        vga::print_str("Error executing program: file not found\n");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    }
}
