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
        let pml4_phys = vmm::clone_kernel_pml4()?;

        let child_task = Task {
            id: slot_idx,
            name: "fork_child",
            rsp: parent.rsp,
            stack_addr: stack_frame,
            state: TaskState::Ready,
            fds: parent.fds,
            program_break: parent.program_break,
            program_break_start: parent.program_break_start,
            cwd: parent.cwd,
            cwd_len: parent.cwd_len,
            parent_id: parent_idx,
            pml4_phys,
        };

        TASKS[slot_idx] = Some(child_task);
        Ok(slot_idx)
    } else {
        Err("Scheduler Error: Parent task invalid")
    }
}

/// Terminate the currently running task.
pub unsafe fn exit_current() {
    core::arch::asm!("cli");
    let idx = CURRENT_TASK_IDX;
    if idx != 0 {
        if let Some(ref mut task) = TASKS[idx] {
            task.state = TaskState::Terminated;
            serial::print_str("Scheduler: Task '");
            serial::print_str(task.name);
            serial::print_str("' exited\n");
        }

        core::arch::asm!("sti");
        loop {
            core::arch::asm!("hlt");
        }
    } else {
        core::arch::asm!("sti");
    }
}

/// Wait for a child task to terminate.
pub unsafe fn wait_for_task(child_id: usize) {
    let current_id = CURRENT_TASK_IDX;

    if let Some(ref mut task) = TASKS[current_id] {
        task.state = TaskState::WaitChild(child_id);
    }

    core::arch::asm!("int 32");
}

/// Preemptive scheduler tick called from PIT timer interrupt.
#[no_mangle]
pub unsafe extern "C" fn schedule_tick(current_rsp: u64) -> u64 {
    vga::handle_timer_tick();

    if !SCHEDULER_INITIALIZED {
        return current_rsp;
    }

    for i in 0..MAX_TASKS {
        if let Some(ref mut task) = TASKS[i] {
            if let TaskState::WaitChild(child_id) = task.state {
                let child_exited = child_id >= MAX_TASKS || TASKS[child_id].is_none();
                if child_exited {
                    task.state = TaskState::Ready;
                }
            }
        }
    }

    let current_idx = CURRENT_TASK_IDX;

    let mut current_terminated = false;
    if let Some(ref mut task) = TASKS[current_idx] {
        if task.state == TaskState::Terminated {
            current_terminated = true;
        } else if task.state == TaskState::Running {
            task.rsp = current_rsp;
            task.state = TaskState::Ready;
        } else if let TaskState::WaitChild(_) = task.state {
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

                if current_terminated {
                    if let Some(ref prev_task) = TASKS[current_idx] {
                        release_all_locks_for_task(prev_task.id);
                        if prev_task.stack_addr != 0 {
                            vmm::free_user_pages(prev_task.pml4_phys, prev_task.program_break);
                            pmm::free_frame(prev_task.stack_addr);
                        }
                    }
                    TASKS[current_idx] = None;
                }

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

    let mut current_runnable = false;
    if let Some(ref task) = TASKS[current_idx] {
        if task.state == TaskState::Running || task.state == TaskState::Ready {
            current_runnable = true;
        }
    }

    if current_runnable && !current_terminated {
        if let Some(ref mut task) = TASKS[current_idx] {
            task.state = TaskState::Running;
        }
        return current_rsp;
    }

    if let Some(ref mut main_task) = TASKS[0] {
        if current_idx != 0 {
            main_task.state = TaskState::Running;
            CURRENT_TASK_IDX = 0;

            if current_terminated {
                if let Some(ref prev_task) = TASKS[current_idx] {
                    release_all_locks_for_task(prev_task.id);
                    if prev_task.stack_addr != 0 {
                        vmm::free_user_pages(prev_task.pml4_phys, prev_task.program_break);
                        pmm::free_frame(prev_task.stack_addr);
                    }
                }
                TASKS[current_idx] = None;
            }

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
                task.state = TaskState::Terminated;
                return Ok(());
            }
        }
    }
    Err("Task PID not found")
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
                TaskState::Running => {
                    vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                    vga::print_str("RUNNING\n");
                }
                TaskState::Ready => {
                    vga::set_color(vga::Color::LightBlue, vga::Color::Black);
                    vga::print_str("READY\n");
                }
                TaskState::Terminated => {
                    vga::set_color(vga::Color::Red, vga::Color::Black);
                    vga::print_str("TERMINATED\n");
                }
                TaskState::WaitChild(_) => {
                    vga::set_color(vga::Color::Magenta, vga::Color::Black);
                    vga::print_str("WAITING\n");
                }
            }
            vga::set_color(vga::Color::White, vga::Color::Black);
        }
    }
    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
}

fn print_decimal(val: u64) {
    let mut buf = [0u8; 20];
    let mut idx = 20;
    let mut temp = val;
    if temp == 0 {
        serial::print_str("0");
        return;
    }
    while temp > 0 {
        idx -= 1;
        buf[idx] = b'0' + (temp % 10) as u8;
        temp /= 10;
    }
    if let Ok(s) = core::str::from_utf8(&buf[idx..]) {
        serial::print_str(s);
    }
}

/// Terminate task by PID with standard signal code.
pub unsafe fn send_signal(pid: usize, sig: u32) -> Result<(), &'static str> {
    if pid >= MAX_TASKS {
        return Err("Invalid process ID");
    }

    if let Some(ref mut task) = TASKS[pid] {
        if pid == 0 {
            return Err("Cannot terminate root kernel shell process");
        }
        match sig {
            2 | 9 | 15 => {
                task.state = TaskState::Terminated;
                TASKS[pid] = None;
                Ok(())
            }
            _ => Err("Unsupported signal type"),
        }
    } else {
        Err("Process ID not found")
    }
}
