// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Process lifecycle, execution, and identity system call handlers.

use keira_fs::elf::loader::load_elf;
use keira_io::vga;
use keira_mem::pmm;
use keira_mem::vmm;
use keira_task::scheduler::{
    fork_current_task, get_current_gid, get_current_uid, set_current_gid, set_current_uid,
    spawn_user, sys_waitpid, wait_for_task, CURRENT_TASK_IDX, TASKS,
};

use crate::user_copy::{
    copy_from_user, copy_to_user, errno_to_ret, read_user_string, validate_user_ptr, EACCES,
    ECHILD, EFAULT, EINVAL, ENOENT, ENOMEM, EPERM,
};

#[cfg(not(test))]
extern "C" {
    fn get_uptime_ms() -> u64;
}

#[cfg(test)]
unsafe fn get_uptime_ms() -> u64 {
    0
}

/// Syscall 1: Print character to VGA console.
pub fn handle_putc(arg1: u64) -> u64 {
    let c = arg1 as u8;
    let slice = [c];
    if let Ok(s) = core::str::from_utf8(&slice) {
        vga::print_str(s);
    }
    0
}

/// Syscall 2: Exit user mode / terminate process.
pub fn handle_exit(arg1: u64) -> u64 {
    let current_idx = unsafe { CURRENT_TASK_IDX };
    let exit_code = arg1 as i32;
    if current_idx != 0 {
        unsafe {
            keira_task::scheduler::exit_current(exit_code);
        }
        0
    } else {
        0xDEADBEEF
    }
}

/// Syscall 3: Sleep current thread for specified milliseconds.
pub fn handle_sleep(arg1: u64) -> u64 {
    let ms = arg1;
    let start = unsafe { get_uptime_ms() };
    #[cfg(not(test))]
    while unsafe { get_uptime_ms() } < start + ms {
        unsafe {
            core::arch::asm!("sti; int 32; cli");
        }
    }
    #[cfg(test)]
    {
        let _ = (ms, start);
    }
    0
}

/// Syscall 4: Retrieve system uptime in milliseconds.
pub fn handle_uptime() -> u64 {
    unsafe { get_uptime_ms() }
}

/// Syscall 5: Execute ELF user program with argument passing.
pub fn handle_exec(arg1: u64, arg2: u64) -> u64 {
    let filename_ptr = arg1 as *const u8;
    let mut name_buf = [0u8; 128];
    let len = match unsafe { read_user_string(filename_ptr, &mut name_buf) } {
        Ok(l) => l,
        Err(e) => return errno_to_ret(e),
    };

    if let Ok(filename_str) = core::str::from_utf8(&name_buf[..len]) {
        let task_id = unsafe { CURRENT_TASK_IDX };
        if !keira_task::security::check_path_access(
            task_id as u64,
            filename_str,
            keira_task::security::MAC_EXEC,
        ) {
            return errno_to_ret(EACCES);
        }

        let mut arg_storage: [[u8; 128]; 16] = [[0u8; 128]; 16];
        let mut arg_slices: [&str; 16] = [""; 16];
        let mut argc = 0;

        if arg2 != 0 {
            #[cfg(target_arch = "x86_64")]
            let ptr_size = 8;
            #[cfg(target_arch = "x86")]
            let ptr_size = 4;

            for (i, storage) in arg_storage.iter_mut().enumerate() {
                let mut ptr_bytes = [0u8; 8];
                if unsafe {
                    copy_from_user(
                        &mut ptr_bytes[..ptr_size],
                        arg2 + (i as u64 * ptr_size as u64),
                    )
                }
                .is_ok()
                {
                    let user_str_ptr = if ptr_size == 8 {
                        u64::from_le_bytes(ptr_bytes) as usize
                    } else {
                        u32::from_le_bytes([ptr_bytes[0], ptr_bytes[1], ptr_bytes[2], ptr_bytes[3]])
                            as usize
                    };
                    if user_str_ptr == 0 {
                        break;
                    }
                    if let Ok(l) = unsafe { read_user_string(user_str_ptr as *const u8, storage) } {
                        if let Ok(s) = core::str::from_utf8(&storage[..l]) {
                            arg_slices[i] = s;
                            argc += 1;
                        }
                    }
                } else {
                    break;
                }
            }
        }

        if argc == 0 {
            arg_slices[0] = filename_str;
            argc = 1;
        }

        unsafe {
            let child_pml4 = match vmm::clone_kernel_pml4() {
                Ok(p) => p,
                Err(_) => return errno_to_ret(ENOMEM),
            };
            let parent_pml4 = vmm::active_pml4();
            vmm::switch_address_space(child_pml4);

            let entry_point = match load_elf(filename_str) {
                Ok(ep) => {
                    let elf_bytes = keira_fs::elf::last_loaded_elf_slice();
                    let _ = keira_crypto::tpm::measure_binary(elf_bytes, filename_str);
                    ep
                }
                Err(_) => {
                    vmm::switch_address_space(parent_pml4);
                    vmm::free_user_pages(child_pml4, 0x600000000000);
                    return errno_to_ret(ENOENT);
                }
            };

            let stack_pages = 256;
            #[cfg(target_arch = "x86_64")]
            let (stack_bottom, top_stack_page): (u64, u64) =
                (0x7FFFFFD80000, 0x7FFFFFE00000 - pmm::PAGE_SIZE);
            #[cfg(target_arch = "x86")]
            let (stack_bottom, top_stack_page): (u64, u64) =
                (0x07F00000, 0x07FFF000 - pmm::PAGE_SIZE);

            for p in 0..stack_pages {
                let page_vaddr = stack_bottom + (p * pmm::PAGE_SIZE);
                if let Some(frame) = pmm::alloc_frame() {
                    let _ = vmm::map_page(
                        page_vaddr,
                        frame,
                        vmm::PAGE_USER | vmm::PAGE_WRITABLE | vmm::PAGE_PRESENT,
                    );
                    let ptr = page_vaddr as *mut u8;
                    core::ptr::write_bytes(ptr, 0, pmm::PAGE_SIZE as usize);
                }
            }

            let ptr = top_stack_page as *mut u8;
            #[cfg(target_arch = "x86_64")]
            let initial_user_rsp = keira_task::stack::setup_user_stack_64(
                ptr,
                top_stack_page,
                &arg_slices[..argc],
                entry_point,
            );
            #[cfg(target_arch = "x86")]
            let initial_user_rsp = keira_task::stack::setup_user_stack_32(
                ptr,
                top_stack_page,
                &arg_slices[..argc],
                entry_point,
            );

            vmm::switch_address_space(parent_pml4);

            match spawn_user("user_app", entry_point, initial_user_rsp, child_pml4) {
                Ok(pid) => pid as u64,
                Err(_) => errno_to_ret(ENOMEM),
            }
        }
    } else {
        errno_to_ret(EINVAL)
    }
}

/// Syscall 13: Wait for specific task termination.
pub fn handle_wait(arg1: u64) -> u64 {
    let child_id = arg1 as usize;
    unsafe {
        wait_for_task(child_id);
    }
    0
}

/// Syscall 14: Get current process identifier.
pub fn handle_getpid() -> u64 {
    unsafe { CURRENT_TASK_IDX as u64 }
}

/// Syscall 15: Get current working directory.
pub fn handle_getcwd(arg1: u64, arg2: u64) -> u64 {
    let buf_ptr = arg1;
    let len = arg2;
    unsafe {
        let task = &TASKS[CURRENT_TASK_IDX];
        if let Some(t) = task {
            let to_copy = (t.cwd_len).min(len as usize);
            if copy_to_user(buf_ptr, &t.cwd[..to_copy]).is_ok() {
                return to_copy as u64;
            }
        }
    }
    errno_to_ret(EFAULT)
}

/// Syscall 16: Change current working directory.
pub fn handle_chdir(arg1: u64) -> u64 {
    let path_ptr = arg1 as *const u8;
    let mut path_buf = [0u8; 128];
    let len = match unsafe { read_user_string(path_ptr, &mut path_buf) } {
        Ok(l) => l,
        Err(e) => return errno_to_ret(e),
    };

    unsafe {
        let task = &mut TASKS[CURRENT_TASK_IDX];
        if let Some(t) = task {
            t.cwd_len = len;
            t.cwd[..len].copy_from_slice(&path_buf[..len]);
            0
        } else {
            errno_to_ret(EFAULT)
        }
    }
}

/// Syscall 30: Fork current process.
pub fn handle_fork() -> u64 {
    unsafe {
        match fork_current_task() {
            Ok(child_pid) => child_pid as u64,
            Err(_) => errno_to_ret(ENOMEM),
        }
    }
}

/// Syscall 41: Clone thread.
pub fn handle_clone_thread() -> u64 {
    unsafe {
        match fork_current_task() {
            Ok(child_pid) => child_pid as u64,
            Err(_) => errno_to_ret(ENOMEM),
        }
    }
}

/// Syscall 59: Process control options (name set/get).
pub fn handle_prctl(arg1: u64, arg2: u64) -> u64 {
    let option = arg1 as i32;
    match option {
        15 => {
            if let Err(e) = unsafe { validate_user_ptr(arg2, 16, false) } {
                return errno_to_ret(e);
            }
            0
        }
        16 => {
            if let Err(e) = unsafe { validate_user_ptr(arg2, 16, true) } {
                return errno_to_ret(e);
            }
            let name = b"keira_task\0";
            if unsafe { copy_to_user(arg2, name) }.is_ok() {
                0
            } else {
                errno_to_ret(EFAULT)
            }
        }
        _ => errno_to_ret(EINVAL),
    }
}

/// Syscall 60: Get current user ID.
pub fn handle_getuid() -> u64 {
    unsafe { get_current_uid() as u64 }
}

/// Syscall 61: Set current user ID.
pub fn handle_setuid(arg1: u64) -> u64 {
    unsafe {
        match set_current_uid(arg1 as u32) {
            Ok(()) => 0,
            Err(_) => errno_to_ret(EPERM),
        }
    }
}

/// Syscall 62: Wait for process status changes.
pub fn handle_waitpid(arg1: u64, arg2: u64, arg3: u64) -> u64 {
    unsafe {
        if arg2 != 0 {
            if arg2 % 4 != 0 {
                return errno_to_ret(EFAULT);
            }
            if let Err(e) = validate_user_ptr(arg2, 4, true) {
                return errno_to_ret(e);
            }
        }
        let mut status: i32 = 0;
        let status_ptr = if arg2 != 0 {
            &mut status as *mut i32
        } else {
            core::ptr::null_mut()
        };
        match sys_waitpid(arg1 as i64, status_ptr, arg3 as i32) {
            Ok(reaped_pid) => {
                if reaped_pid > 0 && arg2 != 0 {
                    if let Err(e) = copy_to_user(arg2, &status.to_ne_bytes()) {
                        return errno_to_ret(e);
                    }
                }
                reaped_pid as u64
            }
            Err(e) => {
                if e == "EINVAL" {
                    errno_to_ret(EINVAL)
                } else if e == "EFAULT" {
                    errno_to_ret(EFAULT)
                } else {
                    errno_to_ret(ECHILD)
                }
            }
        }
    }
}

/// Syscall 63: Get parent process ID.
pub fn handle_getppid() -> u64 {
    unsafe {
        let task = &TASKS[CURRENT_TASK_IDX];
        if let Some(t) = task {
            t.parent_id as u64
        } else {
            0
        }
    }
}

/// Syscall 68: Get current group ID.
pub fn handle_getgid() -> u64 {
    unsafe { get_current_gid() as u64 }
}

/// Syscall 69: Set current group ID.
pub fn handle_setgid(arg1: u64) -> u64 {
    unsafe {
        match set_current_gid(arg1 as u32) {
            Ok(()) => 0,
            Err(_) => errno_to_ret(EPERM),
        }
    }
}
