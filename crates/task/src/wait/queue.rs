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

//! Linux-grade kernel wait queue head implementation.

use keira_core::sync::{IrqSpinLock, LockRank};

use crate::types::{TaskState, MAX_TASKS};
use crate::wait::entry::{WaitQueueEntry, WQ_FLAG_WOKEN};

/// Maximum concurrent waiting tasks per wait queue head.
pub const MAX_WAITERS_PER_QUEUE: usize = 16;

/// Wait queue head representing an event synchronization point.
pub struct WaitQueueHead {
    /// Diagnostic identifier name.
    pub name: &'static str,
    /// Interrupt-safe spinlock protecting waiter list mutations.
    pub lock: IrqSpinLock,
    /// Bounded table of waiting task entries.
    pub waiters: [Option<WaitQueueEntry>; MAX_WAITERS_PER_QUEUE],
    /// Current number of registered waiters.
    pub waiters_count: usize,
    /// Cumulative count of tasks that entered wait state.
    pub total_waits: u64,
    /// Cumulative count of tasks successfully woken.
    pub total_wakeups: u64,
    /// Monotonic timestamp in milliseconds of the last wakeup event.
    pub last_wake_tick: u64,
}

impl WaitQueueHead {
    /// Constructs a new empty wait queue head with the specified diagnostic name.
    pub const fn new(name: &'static str) -> Self {
        Self {
            name,
            lock: IrqSpinLock::with_rank(LockRank::Scheduler),
            waiters: [None; MAX_WAITERS_PER_QUEUE],
            waiters_count: 0,
            total_waits: 0,
            total_wakeups: 0,
            last_wake_tick: 0,
        }
    }

    /// Enqueues a task entry into the wait queue and updates its task state.
    pub fn prepare_to_wait(&mut self, entry: WaitQueueEntry, state: TaskState) {
        let _guard = self.lock.lock();

        for slot in self.waiters.iter() {
            if let Some(ref e) = slot {
                if e.task_id == entry.task_id {
                    return;
                }
            }
        }

        let mut inserted = false;
        for slot in self.waiters.iter_mut() {
            if slot.is_none() {
                *slot = Some(entry);
                self.waiters_count += 1;
                self.total_waits += 1;
                inserted = true;
                break;
            }
        }

        if inserted {
            unsafe {
                let pid = entry.task_id;
                if pid < MAX_TASKS {
                    if let Some(ref mut t) = crate::scheduler::core::TASKS[pid] {
                        t.state = state;
                    }
                    crate::scheduler::work_stealing::mark_task_dequeued(pid);
                }
            }
        }
    }

    /// Removes a task from the wait queue and restores running state if necessary.
    pub fn finish_wait(&mut self, task_id: usize) {
        let _guard = self.lock.lock();

        for slot in self.waiters.iter_mut() {
            if let Some(ref e) = slot {
                if e.task_id == task_id {
                    *slot = None;
                    if self.waiters_count > 0 {
                        self.waiters_count -= 1;
                    }
                    break;
                }
            }
        }

        unsafe {
            if task_id < MAX_TASKS {
                if let Some(ref mut t) = crate::scheduler::core::TASKS[task_id] {
                    if t.state == TaskState::Blocked {
                        t.state = TaskState::Running;
                    }
                }
            }
        }
    }

    /// Wakes up up to `nr` exclusive waiters and all non-exclusive waiters.
    pub fn wake_up_nr(&mut self, nr: usize) -> usize {
        let _guard = self.lock.lock();
        let mut woken = 0usize;
        let mut exclusive_remaining = nr;

        for slot in self.waiters.iter_mut() {
            if let Some(mut entry) = slot.take() {
                let is_exclusive = entry.is_exclusive();
                if is_exclusive && exclusive_remaining == 0 {
                    *slot = Some(entry);
                    continue;
                }

                entry.flags |= WQ_FLAG_WOKEN;
                if self.waiters_count > 0 {
                    self.waiters_count -= 1;
                }
                woken += 1;

                unsafe {
                    let pid = entry.task_id;
                    if pid < MAX_TASKS {
                        let mut should_enqueue = false;
                        if let Some(ref mut t) = crate::scheduler::core::TASKS[pid] {
                            if t.state == TaskState::Blocked {
                                t.state = TaskState::Ready;
                                should_enqueue = pid != 0;
                            }
                        }
                        if should_enqueue {
                            let core_id = keira_arch::cpu::get_current_core_id();
                            crate::scheduler::work_stealing::push_local(core_id, pid);
                        }
                    }
                }

                if is_exclusive && exclusive_remaining > 0 {
                    exclusive_remaining -= 1;
                }
            }
        }

        self.total_wakeups += woken as u64;
        self.last_wake_tick = keira_arch::timers::pit::uptime_ms();
        woken
    }

    /// Wakes up all non-exclusive waiters plus at most one exclusive waiter.
    #[inline]
    pub fn wake_up(&mut self) -> usize {
        self.wake_up_nr(1)
    }

    /// Wakes up all waiters registered in this wait queue head.
    #[inline]
    pub fn wake_up_all(&mut self) -> usize {
        self.wake_up_nr(usize::MAX)
    }

    /// Returns true if no tasks are currently waiting on this queue.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.waiters_count == 0
    }

    /// Returns the number of currently waiting tasks.
    #[inline]
    pub fn waiter_count(&self) -> usize {
        self.waiters_count
    }
}
