// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Preemptive Round-Robin multitasking scheduler, context switching, and task lifecycle management.

use super::types::{FileDescriptor, InterruptContext, Task, TaskState};
use keira_fs::lock::flock::release_all_locks_for_task;
use keira_io::serial;
use keira_io::vga;
use keira_mem::pmm;
use keira_mem::vmm;

extern "C" {
    static mut kernel_stack_temp: u64;
}

pub const MAX_TASKS: usize = 8;

pub static mut TASKS: [Option<Task>; MAX_TASKS] = [None, None, None, None, None, None, None, None];
pub static mut CURRENT_TASK_IDX: usize = 0;
pub static mut SCHEDULER_INITIALIZED: bool = false;

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
        fds: [FileDescriptor::new(); 8],
        program_break: 0,
        program_break_start: 0,
        cwd: main_cwd,
        cwd_len: 1,
        parent_id: 0,
        pml4_phys: boot_pml4,
        exit_code: 0,
        is_user: false,
    };
    TASKS[0] = Some(main_task);
    CURRENT_TASK_IDX = 0;
    SCHEDULER_INITIALIZED = true;
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
        fds: [FileDescriptor::new(); 8],
        program_break: 0,
        program_break_start: 0,
        cwd: child_cwd,
        cwd_len: parent_cwd_len,
        parent_id,
        pml4_phys: parent_pml4,
        exit_code: 0,
        is_user: false,
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
        fds: [FileDescriptor::new(); 8],
        program_break: 0x600000000000,
        program_break_start: 0x600000000000,
        cwd: child_cwd,
        cwd_len: parent_cwd_len,
        parent_id,
        pml4_phys,
        exit_code: 0,
        is_user: true,
    };

    TASKS[slot_idx] = Some(new_task);

    serial::print_str("Scheduler: Spawned user task '");
    serial::print_str(name);
    serial::print_str("' in slot ");
    print_decimal(slot_idx as u64);
    serial::print_str("\n");

    Ok(slot_idx)
}

/// Clones the currently running task into a new child process (fork).
pub unsafe fn fork_current_task() -> Result<usize, &'static str> {
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

        // Copy register context to child stack
        let context_size = core::mem::size_of::<InterruptContext>() as u64;
        let child_context_ptr = (stack_top - context_size) as *mut InterruptContext;

        if parent.rsp != 0 {
            let parent_context = parent.rsp as *const InterruptContext;
            core::ptr::copy_nonoverlapping(parent_context, child_context_ptr, 1);
            // In child process, fork() returns 0 in RAX
            (*child_context_ptr).rax = 0;
        }

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
        };

        TASKS[slot_idx] = Some(child_task);
        Ok(slot_idx)
    } else {
        Err("Scheduler Error: Parent task invalid")
    }
}

/// Terminate the currently running task with an exit code, transitioning to Zombie.
pub unsafe fn exit_current(exit_code: i32) {
    core::arch::asm!("cli");
    let idx = CURRENT_TASK_IDX;
    if idx != 0 {
        if let Some(ref mut task) = TASKS[idx] {
            task.exit_code = exit_code;
            task.state = TaskState::Zombie(exit_code);

            // Wake up parent if blocked
            let parent_id = task.parent_id;
            if parent_id < MAX_TASKS {
                if let Some(ref mut parent) = TASKS[parent_id] {
                    if parent.state == TaskState::Blocked {
                        parent.state = TaskState::Ready;
                    }
                }
            }

            serial::print_str("Scheduler: Task '");
            serial::print_str(task.name);
            serial::print_str("' exited (Zombie)\n");
        }

        core::arch::asm!("sti");
        loop {
            core::arch::asm!("hlt");
        }
    } else {
        core::arch::asm!("sti");
    }
}

/// Wait for a child process to change state (waitpid), reaping zombies with safe pointer validation.
pub unsafe fn sys_waitpid(
    target_pid: i64,
    status_ptr: *mut i32,
    options: u32,
) -> Result<usize, &'static str> {
    if options != 0 {
        return Err("EINVAL");
    }

    if !status_ptr.is_null() {
        let ptr_val = status_ptr as u64;
        if ptr_val < 0x10000
            || ptr_val > 0x0000_7FFF_FFFF_FFFF
            || !vmm::is_user_page_mapped(ptr_val, true)
        {
            return Err("EFAULT");
        }
    }

    let parent_idx = CURRENT_TASK_IDX;

    // 1. Search for matching zombie child
    for i in 1..MAX_TASKS {
        if let Some(ref child) = TASKS[i] {
            if child.parent_id == parent_idx {
                if target_pid == -1 || child.id == target_pid as usize {
                    if let TaskState::Zombie(code) = child.state {
                        let reaped_id = child.id;
                        if !status_ptr.is_null() {
                            *status_ptr = code;
                        }

                        // Release child locks and memory
                        release_all_locks_for_task(reaped_id);
                        if child.stack_addr != 0 {
                            vmm::free_user_pages(child.pml4_phys, child.program_break);
                            pmm::free_frame(child.stack_addr);
                        }
                        TASKS[i] = None;
                        return Ok(reaped_id);
                    }
                }
            }
        }
    }

    // 2. Check if any matching child is still alive
    let mut has_living_child = false;
    for i in 1..MAX_TASKS {
        if let Some(ref child) = TASKS[i] {
            if child.parent_id == parent_idx
                && (target_pid == -1 || child.id == target_pid as usize)
            {
                has_living_child = true;
                break;
            }
        }
    }

    if !has_living_child {
        return Err("No child processes");
    }

    // 3. Block parent until a child exits
    if let Some(ref mut parent) = TASKS[parent_idx] {
        parent.state = TaskState::Blocked;
    }

    core::arch::asm!("int 32");

    // Retry reaping after waking up
    for i in 1..MAX_TASKS {
        if let Some(ref child) = TASKS[i] {
            if child.parent_id == parent_idx {
                if target_pid == -1 || child.id == target_pid as usize {
                    if let TaskState::Zombie(code) = child.state {
                        let reaped_id = child.id;
                        if !status_ptr.is_null() {
                            *status_ptr = code;
                        }
                        release_all_locks_for_task(reaped_id);
                        if child.stack_addr != 0 {
                            vmm::free_user_pages(child.pml4_phys, child.program_break);
                            pmm::free_frame(child.stack_addr);
                        }
                        TASKS[i] = None;
                        return Ok(reaped_id);
                    }
                }
            }
        }
    }

    Err("Interrupted wait")
}

/// Wait for a child task to terminate.
pub unsafe fn wait_for_task(child_id: usize) {
    let _ = sys_waitpid(child_id as i64, core::ptr::null_mut(), 0);
}

/// Preemptive scheduler tick called from PIT timer interrupt.
#[no_mangle]
pub unsafe extern "C" fn schedule_tick(current_rsp: u64) -> u64 {
    vga::handle_timer_tick();

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
                    kernel_stack_temp = task.stack_addr + pmm::PAGE_SIZE;
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
    if pid == 0 {
        return Err("Signal delivery to bootstrap kernel shell is restricted");
    }

    if let Some(ref mut task) = TASKS[pid] {
        match sig {
            9 | 15 => {
                task.state = TaskState::Zombie(sig as i32);
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
            _ => Err("Unsupported or invalid POSIX signal number"),
        }
    } else {
        Err("Process with specified PID does not exist")
    }
}

/// List all registered tasks.
pub unsafe fn list_tasks() {
    vga::set_color(vga::Color::LightBlue, vga::Color::Black);
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
                    vga::set_color(vga::Color::LightBlue, vga::Color::Black);
                    vga::print_str("READY\n");
                }
                TaskState::Blocked => {
                    vga::set_color(vga::Color::Magenta, vga::Color::Black);
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
