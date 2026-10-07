// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Preemptive timer tick dispatch, lock-free work-stealing, and CPU context switching.

use core::sync::atomic::Ordering;
use keira_io::vga;
use keira_mem::{pmm, vmm};

use crate::scheduler::core::{
    get_boot_kernel_stack, main_kernel_stack, set_kernel_stack, CURRENT_TASK_IDX,
    SCHEDULER_INITIALIZED, TASKS, TOTAL_CONTEXT_SWITCHES, TOTAL_SCHEDULER_TICKS,
};
use crate::scheduler::work_stealing::{pick_next_task, push_local};
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

    TOTAL_SCHEDULER_TICKS.fetch_add(1, Ordering::Relaxed);

    let core_id = keira_arch::cpu::get_current_core_id();
    let current_idx = CURRENT_TASK_IDX;

    if let Some(ref mut task) = TASKS[current_idx] {
        task.cpu_ticks += 1;
        if task.state == TaskState::Running || task.state == TaskState::Ready {
            task.rsp = current_rsp;
            task.state = TaskState::Ready;
        } else if task.state == TaskState::Blocked {
            task.rsp = current_rsp;
        }
    }

    let candidate = pick_next_task(core_id, current_idx);

    if let Some(next_idx) = candidate {
        if let Some(ref mut task) = TASKS[next_idx] {
            if task.state == TaskState::Ready {
                if current_idx != 0 && current_idx != next_idx {
                    if let Some(ref cur) = TASKS[current_idx] {
                        if cur.state == TaskState::Ready {
                            push_local(core_id, current_idx);
                        }
                    }
                }

                task.state = TaskState::Running;
                if next_idx != current_idx {
                    TOTAL_CONTEXT_SWITCHES.fetch_add(1, Ordering::Relaxed);
                    task.switches += 1;
                }
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
    }

    if core_id == 0 {
        if let Some(ref mut main_task) = TASKS[0] {
            if main_task.state == TaskState::Ready {
                if current_idx != 0 {
                    if let Some(ref cur) = TASKS[current_idx] {
                        if cur.state == TaskState::Ready {
                            push_local(core_id, current_idx);
                        }
                    }
                }

                main_task.state = TaskState::Running;
                if current_idx != 0 {
                    TOTAL_CONTEXT_SWITCHES.fetch_add(1, Ordering::Relaxed);
                    main_task.switches += 1;
                }
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
    }

    if let Some(ref mut cur) = TASKS[current_idx] {
        if cur.state == TaskState::Ready {
            cur.state = TaskState::Running;
        }
    }

    current_rsp
}
