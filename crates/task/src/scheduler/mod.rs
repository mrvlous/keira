// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Preemptive Round-Robin multitasking scheduler, context switching, and task lifecycle management.

use super::types::{FileDescriptor, InterruptContext, Task, TaskState, MAX_FDS};
use keira_core::sync::{IrqSpinLock, LockRank};
use keira_fs::lock::flock::release_all_locks_for_task;
use keira_io::serial;
use keira_io::vga;
use keira_mem::pmm;
use keira_mem::vmm;

extern "C" {
    static mut main_kernel_stack: u64;
    fn set_kernel_stack(sp0: usize);
    fn get_boot_kernel_stack() -> usize;
}

pub const MAX_TASKS: usize = 64;

pub static SCHEDULER_LOCK: IrqSpinLock = IrqSpinLock::with_rank(LockRank::Scheduler);
pub static mut TASKS: [Option<Task>; MAX_TASKS] = [const { None }; MAX_TASKS];
pub static mut CURRENT_TASK_IDX: usize = 0;
pub static mut SCHEDULER_INITIALIZED: bool = false;

pub type TaskResourceCleanupHook = fn(pid: usize);
static mut TASK_CLEANUP_HOOK: Option<TaskResourceCleanupHook> = None;

/// Register kernel-level resource cleanup callback (invoked on task exit and zombie reaping).
pub fn register_task_cleanup_hook(hook: TaskResourceCleanupHook) {
    unsafe {
        TASK_CLEANUP_HOOK = Some(hook);
    }
}

/// Initialize the scheduler and register the bootstrap thread as Task 0.
pub unsafe fn init() {
    let mut main_cwd = [0u8; 128];
    main_cwd[0] = b'/';
    let boot_pml4 = vmm::active_pml4();
    let main_task = Task {
        id: 0,
        name: "kernel_shell",
        rsp: 0,
        stack_addr: 0,
        state: TaskState::Running,
        fds: [FileDescriptor::new(); MAX_FDS],
        program_break: 0,
        program_break_start: 0,
        cwd: main_cwd,
        cwd_len: 1,
        parent_id: 0,
        pml4_phys: boot_pml4,
        exit_code: 0,
        is_user: false,
        uid: 0,
        gid: 0,
        euid: 0,
        egid: 0,
        saved_sigcontext: None,
        signal_mask: 0,
        pending_signals: 0,
        is_orphan: false,
    };
    TASKS[0] = Some(main_task);
    CURRENT_TASK_IDX = 0;
    SCHEDULER_INITIALIZED = true;
    keira_fs::proc::register_task_hooks(
        task_status_provider,
        task_cmdline_provider,
        current_pid_provider,
    );
}

/// Dynamic task status provider for /system/proc/[pid]/status.
pub fn task_status_provider(pid: usize, buf: &mut [u8]) -> Option<usize> {
    unsafe {
        if pid >= MAX_TASKS {
            return None;
        }
        let task = TASKS[pid].as_ref()?;
        let mut open_fds = 0;
        for fd in &task.fds {
            if fd.is_open {
                open_fds += 1;
            }
        }
        let state_str = match task.state {
            TaskState::Running => "R (running)",
            TaskState::Ready => "S (sleeping)",
            TaskState::Blocked => "D (disk sleep)",
            TaskState::Zombie(_) | TaskState::Exited(_) => "Z (zombie)",
            TaskState::Created => "I (idle)",
        };

        use core::fmt::Write;
        struct SliceWriter<'a> {
            buf: &'a mut [u8],
            offset: usize,
        }
        impl<'a> Write for SliceWriter<'a> {
            fn write_str(&mut self, s: &str) -> core::fmt::Result {
                let bytes = s.as_bytes();
                let avail = self.buf.len().saturating_sub(self.offset);
                let to_write = bytes.len().min(avail);
                self.buf[self.offset..self.offset + to_write].copy_from_slice(&bytes[..to_write]);
                self.offset += to_write;
                Ok(())
            }
        }

        let mut writer = SliceWriter { buf, offset: 0 };
        let _ = core::write!(
            writer,
            "Name:\t{}\n\
             State:\t{}\n\
             Tgid:\t{}\n\
             Pid:\t{}\n\
             PPid:\t{}\n\
             Uid:\t{}\t{}\n\
             Gid:\t{}\t{}\n\
             FDSize:\t{}\n\
             SigBlk:\t{:08x}\n\
             SigPnd:\t{:08x}\n",
            task.name,
            state_str,
            task.id,
            task.id,
            task.parent_id,
            task.uid,
            task.euid,
            task.gid,
            task.egid,
            open_fds,
            task.signal_mask,
            task.pending_signals
        );
        Some(writer.offset)
    }
}

/// Dynamic task cmdline provider for /system/proc/[pid]/cmdline.
pub fn task_cmdline_provider(pid: usize, buf: &mut [u8]) -> Option<usize> {
    unsafe {
        if pid >= MAX_TASKS {
            return None;
        }
        let task = TASKS[pid].as_ref()?;
        let bytes = task.name.as_bytes();
        let to_copy = bytes.len().min(buf.len());
        buf[..to_copy].copy_from_slice(&bytes[..to_copy]);
        Some(to_copy)
    }
}

/// Dynamic provider for active task PID.
pub fn current_pid_provider() -> usize {
    unsafe { CURRENT_TASK_IDX }
}

/// Spawn a new kernel thread.
pub unsafe fn spawn(name: &'static str, entry_point: fn()) -> Result<usize, &'static str> {
    let mut slot = None;
    for i in 0..MAX_TASKS {
        if TASKS[i].is_none() {
            slot = Some(i);
            break;
        }
    }

    if slot.is_none() {
        reap_orphaned_zombies();
        for i in 0..MAX_TASKS {
            if TASKS[i].is_none() {
                slot = Some(i);
                break;
            }
        }
    }

    let slot_idx = slot.ok_or("Scheduler: Maximum task limit reached")?;

    let stack_frame = pmm::alloc_frame().ok_or("Scheduler: Out of memory for task stack")?;
    let stack_top = stack_frame + pmm::PAGE_SIZE;

    let context_ptr =
        (stack_top - core::mem::size_of::<InterruptContext>() as u64) as *mut InterruptContext;

    (*context_ptr).r15 = 0;
    (*context_ptr).r14 = 0;
    (*context_ptr).r13 = 0;
    (*context_ptr).r12 = 0;
    (*context_ptr).r11 = 0;
    (*context_ptr).r10 = 0;
    (*context_ptr).r9 = 0;
    (*context_ptr).r8 = 0;
    (*context_ptr).rdi = 0;
    (*context_ptr).rsi = 0;
    (*context_ptr).rbp = 0;
    (*context_ptr).rbx = 0;
    (*context_ptr).rdx = 0;
    (*context_ptr).rcx = 0;
    (*context_ptr).rax = 0;

    (*context_ptr).rip = entry_point as usize as u64;
    (*context_ptr).cs = 0x08;
    (*context_ptr).rflags = 0x202;
    (*context_ptr).rsp = stack_top;
    (*context_ptr).ss = 0x10;

    let mut child_cwd = [0u8; 128];
    child_cwd[0] = b'/';
    let mut parent_cwd_len = 1usize;
    let mut parent_pml4 = vmm::active_pml4();
    let parent_id = CURRENT_TASK_IDX;
    if let Some(ref parent) = TASKS[parent_id] {
        child_cwd[..parent.cwd_len].copy_from_slice(&parent.cwd[..parent.cwd_len]);
        parent_cwd_len = parent.cwd_len;
        parent_pml4 = parent.pml4_phys;
    }
    let new_task = Task {
        id: slot_idx,
        name,
        rsp: context_ptr as u64,
        stack_addr: stack_frame,
        state: TaskState::Ready,
        fds: [FileDescriptor::new(); MAX_FDS],
        program_break: 0,
        program_break_start: 0,
        cwd: child_cwd,
        cwd_len: parent_cwd_len,
        parent_id,
        pml4_phys: parent_pml4,
        exit_code: 0,
        is_user: false,
        uid: 0,
        gid: 0,
        euid: 0,
        egid: 0,
        saved_sigcontext: None,
        signal_mask: 0,
        pending_signals: 0,
        is_orphan: false,
    };

    TASKS[slot_idx] = Some(new_task);

    serial::print_str("Scheduler: Spawned task '");
    serial::print_str(name);
    serial::print_str("' in slot ");
    print_decimal(slot_idx as u64);
    serial::print_str("\n");

    Ok(slot_idx)
}

/// Spawn a new user-space Ring 3 task.
pub unsafe fn spawn_user(
    name: &'static str,
    entry_point: u64,
    user_rsp: u64,
    pml4_phys: u64,
) -> Result<usize, &'static str> {
    let mut slot = None;
    for i in 0..MAX_TASKS {
        if TASKS[i].is_none() {
            slot = Some(i);
            break;
        }
    }

    if slot.is_none() {
        reap_orphaned_zombies();
        for i in 0..MAX_TASKS {
            if TASKS[i].is_none() {
                slot = Some(i);
                break;
            }
        }
    }

    let slot_idx = slot.ok_or("Scheduler: Maximum task limit reached")?;

    let stack_frame = pmm::alloc_frame().ok_or("Scheduler: Out of memory for task stack")?;
    let stack_top = stack_frame + pmm::PAGE_SIZE;

    let context_ptr =
        (stack_top - core::mem::size_of::<InterruptContext>() as u64) as *mut InterruptContext;

    (*context_ptr).r15 = 0;
    (*context_ptr).r14 = 0;
    (*context_ptr).r13 = 0;
    (*context_ptr).r12 = 0;
    (*context_ptr).r11 = 0;
    (*context_ptr).r10 = 0;
    (*context_ptr).r9 = 0;
    (*context_ptr).r8 = 0;
    (*context_ptr).rdi = 0;
    (*context_ptr).rsi = 0;
    (*context_ptr).rbp = 0;
    (*context_ptr).rbx = 0;
    (*context_ptr).rdx = 0;
    (*context_ptr).rcx = 0;
    (*context_ptr).rax = 0;

    (*context_ptr).rip = entry_point;
    (*context_ptr).cs = 0x2B;
    (*context_ptr).rflags = 0x202;
    (*context_ptr).rsp = user_rsp;
    (*context_ptr).ss = 0x23;

    let mut child_cwd = [0u8; 128];
    child_cwd[0] = b'/';
    let mut parent_cwd_len = 1usize;
    let parent_id = CURRENT_TASK_IDX;
    if let Some(ref parent) = TASKS[parent_id] {
        child_cwd[..parent.cwd_len].copy_from_slice(&parent.cwd[..parent.cwd_len]);
        parent_cwd_len = parent.cwd_len;
    }

    let new_task = Task {
        id: slot_idx,
        name,
        rsp: context_ptr as u64,
        stack_addr: stack_frame,
        state: TaskState::Ready,
        fds: [FileDescriptor::new(); MAX_FDS],
        program_break: 0x600000000000,
        program_break_start: 0x600000000000,
        cwd: child_cwd,
        cwd_len: parent_cwd_len,
        parent_id,
        pml4_phys,
        exit_code: 0,
        is_user: true,
        uid: 0,
        gid: 0,
        euid: 0,
        egid: 0,
        saved_sigcontext: None,
        signal_mask: 0,
        pending_signals: 0,
        is_orphan: false,
    };

    TASKS[slot_idx] = Some(new_task);

    serial::print_str("Scheduler: Spawned user task '");
    serial::print_str(name);
    serial::print_str("' in slot ");
    print_decimal(slot_idx as u64);
    serial::print_str("\n");

    Ok(slot_idx)
}

/// Scan for and reap any orphaned processes adopted by PID 0 that have transitioned to Zombie (assumes SCHEDULER_LOCK held).
pub unsafe fn reap_orphaned_zombies_locked() {
    let curr = CURRENT_TASK_IDX;
    for i in 1..MAX_TASKS {
        if i == curr {
            continue;
        }
        if let Some(ref child) = TASKS[i] {
            if child.is_orphan || child.parent_id == 0 {
                if let TaskState::Zombie(_) = child.state {
                    let reaped_id = child.id;
                    release_all_locks_for_task(reaped_id);
                    if let Some(hook) = TASK_CLEANUP_HOOK {
                        hook(reaped_id);
                    }
                    if child.stack_addr != 0 {
                        vmm::free_user_pages(child.pml4_phys, child.program_break);
                        pmm::free_frame(child.stack_addr);
                    } else if child.pml4_phys != 0 {
                        vmm::cleanup_vmas_for_pml4(child.pml4_phys);
                    }
                    TASKS[i] = None;
                }
            }
        }
    }
}

/// Clones the currently running task into a new child process (fork).
pub unsafe fn fork_current_task() -> Result<usize, &'static str> {
    #[cfg(target_arch = "x86")]
    {
        return Err("Scheduler: Fork is unsupported on 32-bit architecture without MMU paging");
    }

    #[cfg(not(target_arch = "x86"))]
    {
        let _guard = SCHEDULER_LOCK.lock();
        let parent_idx = CURRENT_TASK_IDX;

        let mut slot_idx = 0;
        let mut found = false;
        for i in 1..MAX_TASKS {
            if TASKS[i].is_none() {
                slot_idx = i;
                found = true;
                break;
            }
        }

        if !found {
            reap_orphaned_zombies_locked();
            for i in 1..MAX_TASKS {
                if TASKS[i].is_none() {
                    slot_idx = i;
                    found = true;
                    break;
                }
            }
        }

        if !found {
            return Err("Scheduler Error: Max tasks reached");
        }

        let ptr = &raw const TASKS;
        if let Some(ref parent) = (*ptr)[parent_idx] {
            let stack_frame = pmm::alloc_frame().ok_or("Out of memory for child stack")?;
            let stack_top = stack_frame + pmm::PAGE_SIZE;

            // Clone parent address space with full deep-copy of user pages
            let pml4_phys = match vmm::clone_user_address_space(parent.pml4_phys) {
                Ok(p) => p,
                Err(e) => {
                    pmm::free_frame(stack_frame);
                    return Err(e);
                }
            };

            // Populate child register context from user syscall snapshot
            let context_size = core::mem::size_of::<InterruptContext>() as u64;
            let child_context_ptr = (stack_top - context_size) as *mut InterruptContext;

            let percpu = keira_arch::cpu::get_current_percpu();

            (*child_context_ptr).r15 = percpu.user_r15;
            (*child_context_ptr).r14 = percpu.user_r14;
            (*child_context_ptr).r13 = percpu.user_r13;
            (*child_context_ptr).r12 = percpu.user_r12;
            (*child_context_ptr).r11 = 0;
            (*child_context_ptr).r10 = 0;
            (*child_context_ptr).r9 = 0;
            (*child_context_ptr).r8 = 0;
            (*child_context_ptr).rdi = 0;
            (*child_context_ptr).rsi = 0;
            (*child_context_ptr).rbp = percpu.user_rbp;
            (*child_context_ptr).rbx = percpu.user_rbx;
            (*child_context_ptr).rdx = 0;
            (*child_context_ptr).rcx = 0;
            (*child_context_ptr).rax = 0; // In child process, fork() returns 0!

            let child_rip = if percpu.user_rip >= 0x10000 && percpu.user_rip < 0x0000_8000_0000_0000
            {
                percpu.user_rip
            } else {
                0x0000_0000_4000_0000
            };
            let child_rsp = if percpu.user_rsp >= 0x10000 && percpu.user_rsp < 0x0000_8000_0000_0000
            {
                percpu.user_rsp
            } else {
                0x0000_7FFF_FFFF_F000
            };

            (*child_context_ptr).rip = child_rip;
            (*child_context_ptr).cs = 0x2B; // User code selector (RPL=3)
            (*child_context_ptr).rflags = (percpu.user_rflags | 0x202) & !0x100; // IF=1, TF=0
            (*child_context_ptr).rsp = child_rsp;
            (*child_context_ptr).ss = 0x23; // User data selector (RPL=3)

            let child_task = Task {
                id: slot_idx,
                name: "fork_child",
                rsp: child_context_ptr as u64,
                stack_addr: stack_frame,
                state: TaskState::Ready,
                fds: parent.fds,
                program_break: parent.program_break,
                program_break_start: parent.program_break_start,
                cwd: parent.cwd,
                cwd_len: parent.cwd_len,
                parent_id: parent_idx,
                pml4_phys,
                exit_code: 0,
                is_user: parent.is_user,
                uid: parent.uid,
                gid: parent.gid,
                euid: parent.euid,
                egid: parent.egid,
                saved_sigcontext: None,
                signal_mask: parent.signal_mask,
                pending_signals: 0,
                is_orphan: false,
            };

            TASKS[slot_idx] = Some(child_task);
            Ok(slot_idx)
        } else {
            Err("Scheduler Error: Parent task invalid")
        }
    }
}

/// Terminate the currently running task with an exit code, transitioning to Zombie.
pub unsafe fn exit_current(exit_code: i32) {
    let parent_id = {
        let _guard = SCHEDULER_LOCK.lock();
        let idx = CURRENT_TASK_IDX;
        if idx != 0 {
            let pid = if let Some(ref mut task) = TASKS[idx] {
                task.exit_code = exit_code;
                task.state = TaskState::Zombie(exit_code);

                // Auto-reclaim open file descriptors and flock write locks
                for fd in 0..MAX_FDS {
                    if task.fds[fd].is_open {
                        if task.fds[fd].write_mode {
                            if let Ok(path_str) =
                                core::str::from_utf8(&task.fds[fd].path[..task.fds[fd].path_len])
                            {
                                keira_fs::lock::flock::release_lock(path_str, idx);
                            }
                        }
                        task.fds[fd] = FileDescriptor::new();
                    }
                }

                serial::print_str("Scheduler: Task '");
                serial::print_str(task.name);
                serial::print_str("' exited (Zombie)\n");

                task.parent_id
            } else {
                0
            };

            release_all_locks_for_task(idx);

            if let Some(hook) = TASK_CLEANUP_HOOK {
                hook(idx);
            }

            // Reparent any child tasks to PID 0 (kernel_shell / Init)
            for i in 1..MAX_TASKS {
                if let Some(ref mut child) = TASKS[i] {
                    if child.parent_id == idx {
                        child.parent_id = 0;
                        child.is_orphan = true;
                    }
                }
            }

            // Wake up parent if blocked
            if pid < MAX_TASKS {
                if let Some(ref mut parent) = TASKS[pid] {
                    if parent.state == TaskState::Blocked {
                        parent.state = TaskState::Ready;
                    }
                }
            }

            pid
        } else {
            0
        }
    };

    if parent_id != 0 || CURRENT_TASK_IDX != 0 {
        core::arch::asm!("sti; int 32");
        loop {
            core::arch::asm!("hlt");
        }
    } else {
        core::arch::asm!("sti");
    }
}

/// Scan for and reap any orphaned processes adopted by PID 0 that have transitioned to Zombie.
/// Fully reclaims file locks, IPC resources, user address space, page tables, and stack frames.
pub unsafe fn reap_orphaned_zombies() {
    let _guard = SCHEDULER_LOCK.lock();
    reap_orphaned_zombies_locked();
}

/// Wait for a child process to change state (waitpid), reaping zombies with safe pointer validation.
pub unsafe fn sys_waitpid(
    target_pid: i64,
    status_ptr: *mut i32,
    options: u32,
) -> Result<usize, &'static str> {
    if (options & !1) != 0 {
        return Err("EINVAL");
    }

    let parent_idx = CURRENT_TASK_IDX;

    loop {
        let (reaped_id, status_val, has_living_child) = {
            let _guard = SCHEDULER_LOCK.lock();

            // 1. Check if child already exited (Zombie)
            let mut reaped = None;
            for i in 1..MAX_TASKS {
                if let Some(ref child) = TASKS[i] {
                    if child.parent_id == parent_idx {
                        if target_pid == -1 || child.id == target_pid as usize {
                            if let TaskState::Zombie(code) = child.state {
                                let id = child.id;
                                let encoded_status = if code >= 0 {
                                    (code & 0xff) << 8
                                } else {
                                    (-code) & 0x7f
                                };
                                release_all_locks_for_task(id);
                                if let Some(hook) = TASK_CLEANUP_HOOK {
                                    hook(id);
                                }
                                if child.stack_addr != 0 {
                                    vmm::free_user_pages(child.pml4_phys, child.program_break);
                                    pmm::free_frame(child.stack_addr);
                                } else if child.pml4_phys != 0 {
                                    vmm::cleanup_vmas_for_pml4(child.pml4_phys);
                                }
                                TASKS[i] = None;
                                crate::signal::reset_signal_handlers(id);
                                reaped = Some((id, encoded_status));
                                break;
                            }
                        }
                    }
                }
            }

            if let Some((id, st)) = reaped {
                (Some(id), st, true)
            } else {
                // 2. Check if any matching child is still alive
                let mut living = false;
                for i in 1..MAX_TASKS {
                    if let Some(ref child) = TASKS[i] {
                        if child.parent_id == parent_idx
                            && (target_pid == -1 || child.id == target_pid as usize)
                        {
                            living = true;
                            break;
                        }
                    }
                }
                (None, 0, living)
            }
        };

        if let Some(id) = reaped_id {
            if !status_ptr.is_null() {
                *status_ptr = status_val;
            }
            return Ok(id);
        }

        if !has_living_child {
            return Err("No child processes");
        }

        // Non-blocking wait if WNOHANG is set
        if (options & 1) != 0 {
            return Ok(0);
        }

        // 3. Block parent until a child exits
        {
            let _guard = SCHEDULER_LOCK.lock();
            if let Some(ref mut parent) = TASKS[parent_idx] {
                parent.state = TaskState::Blocked;
            }
        }

        core::arch::asm!("sti; int 32; cli");
    }
}

/// Wait for a child task to terminate.
pub unsafe fn wait_for_task(child_id: usize) {
    let _ = sys_waitpid(child_id as i64, core::ptr::null_mut(), 0);
}

/// Preemptive scheduler tick called from PIT timer interrupt.
#[no_mangle]
pub unsafe extern "C" fn schedule_tick(current_rsp: u64) -> u64 {
    vga::handle_timer_tick();
    keira_arch::power::acpi::record_cpu_heartbeat();

    if !SCHEDULER_INITIALIZED {
        return current_rsp;
    }

    let current_idx = CURRENT_TASK_IDX;

    if let Some(ref mut task) = TASKS[current_idx] {
        if task.state == TaskState::Running {
            task.rsp = current_rsp;
            task.state = TaskState::Ready;
        } else if task.state == TaskState::Blocked {
            task.rsp = current_rsp;
        }
    }

    let mut next_idx = current_idx;
    loop {
        next_idx = (next_idx + 1) % MAX_TASKS;
        if let Some(ref mut task) = TASKS[next_idx] {
            if task.state == TaskState::Ready {
                task.state = TaskState::Running;
                CURRENT_TASK_IDX = next_idx;

                vmm::switch_address_space(task.pml4_phys);
                if task.stack_addr != 0 {
                    let kstack = (task.stack_addr + pmm::PAGE_SIZE) as usize;
                    set_kernel_stack(kstack);
                } else {
                    let boot_stack = get_boot_kernel_stack();
                    let kstack = if boot_stack != 0 {
                        boot_stack
                    } else {
                        main_kernel_stack as usize
                    };
                    if kstack != 0 {
                        set_kernel_stack(kstack);
                    }
                }

                return task.rsp;
            }
        }
        if next_idx == current_idx {
            break;
        }
    }

    if let Some(ref mut main_task) = TASKS[0] {
        if current_idx != 0 {
            main_task.state = TaskState::Running;
            CURRENT_TASK_IDX = 0;
            vmm::switch_address_space(main_task.pml4_phys);
            let boot_stack = get_boot_kernel_stack();
            let kstack = if boot_stack != 0 {
                boot_stack
            } else {
                main_kernel_stack as usize
            };
            if kstack != 0 {
                set_kernel_stack(kstack);
            }
            return main_task.rsp;
        }
    }

    current_rsp
}

/// Terminate/stop a task by PID.
pub unsafe fn stop_task(pid: usize) -> Result<(), &'static str> {
    if pid == 0 {
        return Err("Cannot stop the kernel shell (Task 0)");
    }
    for i in 1..MAX_TASKS {
        if let Some(ref mut task) = TASKS[i] {
            if task.id == pid {
                task.state = TaskState::Zombie(-9);
                return Ok(());
            }
        }
    }
    Err("Task PID not found")
}

/// Deliver a POSIX-like signal to a target task PID.
pub unsafe fn send_signal(pid: usize, sig: u32) -> Result<(), &'static str> {
    if pid >= MAX_TASKS {
        return Err("Target PID out of scheduler table range");
    }
    if let Some(ref mut task) = TASKS[pid] {
        let is_unblockable = sig == 9 || sig == 19;
        if !is_unblockable && (task.signal_mask & (1 << sig)) != 0 {
            task.pending_signals |= 1 << sig;
            return Ok(());
        }

        let handler = super::signal::get_signal_handler(pid, sig);
        if handler != 0 {
            if task.saved_sigcontext.is_none() {
                let mut saved_ctx = InterruptContext::default();
                saved_ctx.rip = task.rsp;
                task.saved_sigcontext = Some(saved_ctx);
            }
            return Ok(());
        }

        if pid == 0 {
            return Err("Signal delivery to bootstrap kernel shell is restricted");
        }

        match sig {
            1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12 | 13 | 14 | 15 => {
                task.state = TaskState::Zombie(-(sig as i32));
                Ok(())
            }
            18 => {
                if task.state == TaskState::Blocked {
                    task.state = TaskState::Ready;
                }
                Ok(())
            }
            19 => {
                task.state = TaskState::Blocked;
                Ok(())
            }
            _ => Ok(()),
        }
    } else {
        Err("Process with specified PID does not exist")
    }
}

/// List all registered tasks.
pub unsafe fn list_tasks() {
    vga::set_color(vga::Color::White, vga::Color::Black);
    vga::print_str("PID    TASK NAME             STATE\n");
    vga::set_color(vga::Color::White, vga::Color::Black);
    for i in 0..MAX_TASKS {
        if let Some(ref task) = TASKS[i] {
            vga::print_u64(task.id as u64);
            let mut pid_len = 0;
            let mut temp = task.id;
            if temp == 0 {
                pid_len = 1;
            } else {
                while temp > 0 {
                    pid_len += 1;
                    temp /= 10;
                }
            }
            for _ in 0..(7 - pid_len) {
                vga::print_str(" ");
            }

            vga::print_str(task.name);
            for _ in 0..(22 - task.name.len()) {
                vga::print_str(" ");
            }

            match task.state {
                TaskState::Created => {
                    vga::set_color(vga::Color::Yellow, vga::Color::Black);
                    vga::print_str("CREATED\n");
                }
                TaskState::Running => {
                    vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                    vga::print_str("RUNNING\n");
                }
                TaskState::Ready => {
                    vga::set_color(vga::Color::White, vga::Color::Black);
                    vga::print_str("READY\n");
                }
                TaskState::Blocked => {
                    vga::set_color(vga::Color::White, vga::Color::Black);
                    vga::print_str("BLOCKED\n");
                }
                TaskState::Exited(c) | TaskState::Zombie(c) => {
                    vga::set_color(vga::Color::Red, vga::Color::Black);
                    vga::print_str("ZOMBIE (exit ");
                    vga::print_u64(c as u64);
                    vga::print_str(")\n");
                }
            }
        }
    }
}

unsafe fn print_decimal(mut val: u64) {
    if val == 0 {
        serial::print_str("0");
        return;
    }
    let mut buf = [0u8; 20];
    let mut idx = 0;
    while val > 0 {
        buf[idx] = b'0' + (val % 10) as u8;
        val /= 10;
        idx += 1;
    }
    while idx > 0 {
        idx -= 1;
        let s = [buf[idx]];
        if let Ok(st) = core::str::from_utf8(&s) {
            serial::print_str(st);
        }
    }
}

/// Query the real UID of the current task.
pub unsafe fn get_current_uid() -> u32 {
    if let Some(ref task) = TASKS[CURRENT_TASK_IDX] {
        task.uid
    } else {
        0
    }
}

/// Query the effective UID of the current task.
pub unsafe fn get_current_euid() -> u32 {
    if let Some(ref task) = TASKS[CURRENT_TASK_IDX] {
        task.euid
    } else {
        0
    }
}

/// Set the real and effective UID of the current task according to POSIX privilege rules.
pub unsafe fn set_current_uid(new_uid: u32) -> Result<(), &'static str> {
    if let Some(ref mut task) = TASKS[CURRENT_TASK_IDX] {
        if task.euid == 0 {
            task.uid = new_uid;
            task.euid = new_uid;
            Ok(())
        } else if new_uid == task.uid {
            task.euid = new_uid;
            Ok(())
        } else {
            Err("Operation not permitted")
        }
    } else {
        Err("No active task")
    }
}

/// Query the real GID of the current task.
pub unsafe fn get_current_gid() -> u32 {
    if let Some(ref task) = TASKS[CURRENT_TASK_IDX] {
        task.gid
    } else {
        0
    }
}

/// Query the effective GID of the current task.
pub unsafe fn get_current_egid() -> u32 {
    if let Some(ref task) = TASKS[CURRENT_TASK_IDX] {
        task.egid
    } else {
        0
    }
}

/// Set the real and effective GID of the current task according to POSIX privilege rules.
pub unsafe fn set_current_gid(new_gid: u32) -> Result<(), &'static str> {
    if let Some(ref mut task) = TASKS[CURRENT_TASK_IDX] {
        if task.euid == 0 {
            task.gid = new_gid;
            task.egid = new_gid;
            Ok(())
        } else if new_gid == task.gid {
            task.egid = new_gid;
            Ok(())
        } else {
            Err("Operation not permitted")
        }
    } else {
        Err("No active task")
    }
}

/// Store a saved interrupt context for signal return in the current task.
pub unsafe fn set_saved_sigcontext(ctx: InterruptContext) {
    if let Some(ref mut task) = TASKS[CURRENT_TASK_IDX] {
        task.saved_sigcontext = Some(ctx);
    }
}

/// Retrieve and clear the saved interrupt context for signal return in the current task.
pub unsafe fn take_saved_sigcontext() -> Option<InterruptContext> {
    if let Some(ref mut task) = TASKS[CURRENT_TASK_IDX] {
        task.saved_sigcontext.take()
    } else {
        None
    }
}

pub const SIG_BLOCK: i32 = 0;
pub const SIG_UNBLOCK: i32 = 1;
pub const SIG_SETMASK: i32 = 2;

/// Get current process signal mask.
pub unsafe fn get_current_signal_mask() -> u32 {
    if let Some(ref t) = TASKS[CURRENT_TASK_IDX] {
        t.signal_mask
    } else {
        0
    }
}

/// Get current process pending signals bitmask.
pub unsafe fn get_current_pending_signals() -> u32 {
    if let Some(ref t) = TASKS[CURRENT_TASK_IDX] {
        t.pending_signals
    } else {
        0
    }
}

/// Modify or inspect process signal mask (Syscall 81: sys_sigprocmask).
pub unsafe fn sys_sigprocmask(how: i32, set: u32, old_set: *mut u32) -> Result<u32, &'static str> {
    if let Some(ref mut task) = TASKS[CURRENT_TASK_IDX] {
        if !old_set.is_null() {
            *old_set = task.signal_mask;
        }

        let unblockable = (1 << 9) | (1 << 19);
        let clean_set = set & !unblockable;

        match how {
            SIG_BLOCK => {
                task.signal_mask |= clean_set;
            }
            SIG_UNBLOCK => {
                task.signal_mask &= !clean_set;
            }
            SIG_SETMASK => {
                task.signal_mask = clean_set;
            }
            _ => return Err("Invalid how parameter for sigprocmask"),
        }

        let unmasked_pending = task.pending_signals & !task.signal_mask;
        if unmasked_pending != 0 {
            for s in 1..32 {
                if (unmasked_pending & (1 << s)) != 0 {
                    task.pending_signals &= !(1 << s);
                    let _ = send_signal(CURRENT_TASK_IDX, s);
                    break;
                }
            }
        }
        Ok(0)
    } else {
        Err("Current task invalid")
    }
}
