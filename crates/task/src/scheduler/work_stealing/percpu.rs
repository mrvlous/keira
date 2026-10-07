// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Per-CPU runqueues and decentralized lock-free task management.
//!
//! Provides static array of Chase-Lev deques mapped to symmetric multiprocessing
//! (SMP) cores, eliminating centralized scheduler spinlock bottlenecks.

use core::sync::atomic::{AtomicBool, Ordering};

use super::deque::ChaseLevDeque;
use crate::types::MAX_TASKS;

/// Maximum number of symmetric multiprocessing (SMP) CPU cores supported.
pub const MAX_CPU_CORES: usize = 16;

/// Per-CPU lock-free Chase-Lev runqueue array.
pub static CPU_RUNQUEUES: [ChaseLevDeque; MAX_CPU_CORES] =
    [const { ChaseLevDeque::new() }; MAX_CPU_CORES];

/// Bitset tracking whether each task descriptor index is currently queued.
pub static TASK_QUEUED: [AtomicBool; MAX_TASKS] = [const { AtomicBool::new(false) }; MAX_TASKS];

/// Enqueues a task into the specified CPU core's lock-free runqueue.
///
/// Returns `true` if the task was successfully transitioned and enqueued.
pub fn enqueue_task(core_id: usize, task_idx: usize) -> bool {
    if task_idx == 0 || task_idx >= MAX_TASKS {
        return false;
    }

    if TASK_QUEUED[task_idx]
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Relaxed)
        .is_err()
    {
        return false;
    }

    let target_core = core_id % MAX_CPU_CORES;
    if CPU_RUNQUEUES[target_core].push(task_idx).is_ok() {
        true
    } else {
        TASK_QUEUED[task_idx].store(false, Ordering::Release);
        false
    }
}

/// Enqueues a task into the local CPU's runqueue.
#[inline]
pub fn push_local(core_id: usize, task_idx: usize) -> bool {
    enqueue_task(core_id, task_idx)
}

/// Pops a task descriptor index from the local CPU core's runqueue.
pub fn pop_local(core_id: usize) -> Option<usize> {
    let target_core = core_id % MAX_CPU_CORES;
    let task_idx = CPU_RUNQUEUES[target_core].pop()?;
    if task_idx < MAX_TASKS {
        TASK_QUEUED[task_idx].store(false, Ordering::Release);
    }
    Some(task_idx)
}

/// Steals a task descriptor index from a remote victim CPU core's runqueue.
pub fn steal_from(victim_core: usize) -> Option<usize> {
    let target_core = victim_core % MAX_CPU_CORES;
    let task_idx = CPU_RUNQUEUES[target_core].steal()?;
    if task_idx < MAX_TASKS {
        TASK_QUEUED[task_idx].store(false, Ordering::Release);
    }
    Some(task_idx)
}

/// Clears the queued status bit for a task descriptor.
pub fn mark_task_dequeued(task_idx: usize) {
    if task_idx < MAX_TASKS {
        TASK_QUEUED[task_idx].store(false, Ordering::Release);
    }
}

/// Checks whether a task descriptor is currently marked as queued in a runqueue.
pub fn is_task_queued(task_idx: usize) -> bool {
    if task_idx < MAX_TASKS {
        TASK_QUEUED[task_idx].load(Ordering::Acquire)
    } else {
        false
    }
}

/// Retrieves the current length of a specific CPU core's runqueue.
pub fn runqueue_len(core_id: usize) -> usize {
    CPU_RUNQUEUES[core_id % MAX_CPU_CORES].len()
}
