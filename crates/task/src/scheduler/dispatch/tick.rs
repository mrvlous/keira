// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Preemptive timer tick dispatch and round-robin CPU context switching.

use keira_io::vga;
use keira_mem::{pmm, vmm};

use crate::scheduler::core::{
    get_boot_kernel_stack, main_kernel_stack, set_kernel_stack, CURRENT_TASK_IDX, MAX_TASKS,
    SCHEDULER_INITIALIZED, TASKS,
};
use crate::types::TaskState;

/// Preemptive scheduler tick called from PIT timer interrupt.
///
/// # Safety
/// Caller must ensure that CPU execution context is saved and `current_rsp` points to a valid interrupt frame.
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
