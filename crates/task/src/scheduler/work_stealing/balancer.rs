// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Decentralized work-stealing task balancer and next-task selection.
//!
//! Provides lock-free task selection: each core attempts to execute from its
//! local Chase-Lev runqueue before initiating work-stealing against remote cores.

use super::percpu::{is_task_queued, pop_local, steal_from, MAX_CPU_CORES};
use crate::scheduler::core::TASKS;
use crate::types::{TaskState, MAX_TASKS};

/// Inspects whether a specific task descriptor index is currently valid and ready.
#[inline]
pub fn is_task_ready(task_idx: usize) -> bool {
    if task_idx == 0 || task_idx >= MAX_TASKS {
        return false;
    }
    unsafe {
        let ptr = &raw const TASKS;
        if let Some(ref task) = (*ptr)[task_idx] {
            task.state == TaskState::Ready
        } else {
            false
        }
    }
}

/// Selects the next runnable task for execution on the specified CPU core.
///
/// Execution order:
/// 1. Pop from local CPU core's Chase-Lev runqueue.
/// 2. Steal from remote CPU cores' runqueues via lock-free CAS.
/// 3. Scan for any ready unqueued tasks excluding the current yielding task.
pub fn pick_next_task(core_id: usize, current_task: usize) -> Option<usize> {
    // 1. Check local runqueue
    while let Some(task_idx) = pop_local(core_id) {
        if is_task_ready(task_idx) {
            return Some(task_idx);
        }
    }

    // 2. Steal work from remote cores
    for offset in 1..MAX_CPU_CORES {
        let victim = (core_id + offset) % MAX_CPU_CORES;
        while let Some(task_idx) = steal_from(victim) {
            if is_task_ready(task_idx) {
                return Some(task_idx);
            }
        }
    }

    // 3. Graceful fallback for unqueued ready tasks (excluding current task)
    (1..MAX_TASKS).find(|&i| i != current_task && is_task_ready(i) && !is_task_queued(i))
}
