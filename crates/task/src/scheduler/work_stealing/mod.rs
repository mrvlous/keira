// SPDX-License-Identifier: GPL-2.0-only
/*
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra <mrvlous@proton.me>
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 */

//! Per-CPU Lock-Free Work-Stealing Scheduler Architecture.
//!
//! Replaces coarse-grained global scheduler spinlocks with per-CPU Chase-Lev
//! deques and decentralized, lock-free work-stealing guarded by Epoch-Based
//! Reclamation (EBR).

pub mod balancer;
pub mod deque;
pub mod percpu;

#[cfg(test)]
mod tests;

pub use balancer::{is_task_ready, pick_next_task};
pub use deque::{ChaseLevDeque, DEQUE_CAPACITY, DEQUE_MASK};
pub use percpu::{
    enqueue_task, is_task_queued, mark_task_dequeued, pop_local, push_local, runqueue_len,
    steal_from, CPU_RUNQUEUES, MAX_CPU_CORES, TASK_QUEUED,
};
