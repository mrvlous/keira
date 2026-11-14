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

//! Work-stealing scheduler queue balancer and per-CPU runqueues.

pub mod balancer {
    use super::percpu::{is_task_queued, pop_local, steal_from, MAX_CPU_CORES};
    use crate::scheduler::core::TASKS;
    use crate::types::{TaskState, MAX_TASKS};

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
}

pub mod deque {
    use core::sync::atomic::{fence, AtomicUsize, Ordering};

    pub const DEQUE_CAPACITY: usize = 64;

    pub const DEQUE_MASK: usize = DEQUE_CAPACITY - 1;

    pub struct ChaseLevDeque {
        top: AtomicUsize,
        bottom: AtomicUsize,
        buffer: [AtomicUsize; DEQUE_CAPACITY],
    }

    impl Default for ChaseLevDeque {
        fn default() -> Self {
            Self::new()
        }
    }

    impl ChaseLevDeque {
        /// Constructs an empty Chase-Lev work-stealing deque.
        pub const fn new() -> Self {
            Self {
                top: AtomicUsize::new(0),
                bottom: AtomicUsize::new(0),
                buffer: [const { AtomicUsize::new(0) }; DEQUE_CAPACITY],
            }
        }

        /// Pushes a task descriptor index onto the bottom of the deque.
        ///
        /// # Safety
        /// Must only be called by the owning CPU core of this runqueue.
        pub fn push(&self, task_idx: usize) -> Result<(), usize> {
            let b = self.bottom.load(Ordering::Relaxed);
            let t = self.top.load(Ordering::Acquire);
            if (b.wrapping_sub(t) as isize) >= DEQUE_CAPACITY as isize {
                return Err(task_idx);
            }

            self.buffer[b & DEQUE_MASK].store(task_idx, Ordering::Relaxed);
            self.bottom.store(b.wrapping_add(1), Ordering::Release);
            Ok(())
        }

        /// Pops a task descriptor index from the bottom of the deque.
        ///
        /// # Safety
        /// Must only be called by the owning CPU core of this runqueue.
        pub fn pop(&self) -> Option<usize> {
            let b = self.bottom.load(Ordering::Relaxed);
            let t = self.top.load(Ordering::Relaxed);
            if (b.wrapping_sub(t) as isize) <= 0 {
                return None;
            }

            let new_b = b.wrapping_sub(1);
            self.bottom.store(new_b, Ordering::Relaxed);
            fence(Ordering::SeqCst);

            let t = self.top.load(Ordering::SeqCst);
            let size = new_b.wrapping_sub(t) as isize;

            if size < 0 {
                self.bottom.store(b, Ordering::Relaxed);
                None
            } else if size == 0 {
                let task_idx = self.buffer[new_b & DEQUE_MASK].load(Ordering::Relaxed);
                if self
                    .top
                    .compare_exchange(t, t.wrapping_add(1), Ordering::SeqCst, Ordering::Relaxed)
                    .is_ok()
                {
                    self.bottom.store(t.wrapping_add(1), Ordering::Relaxed);
                    Some(task_idx)
                } else {
                    self.bottom.store(t.wrapping_add(1), Ordering::Relaxed);
                    None
                }
            } else {
                let task_idx = self.buffer[new_b & DEQUE_MASK].load(Ordering::Relaxed);
                Some(task_idx)
            }
        }

        /// Concurrently steals a task descriptor index from the top of the deque.
        ///
        /// Can be called concurrently by any remote CPU core without acquiring locks.
        pub fn steal(&self) -> Option<usize> {
            let _guard = keira_core::sync::ebr::pin();
            let t = self.top.load(Ordering::Acquire);
            fence(Ordering::SeqCst);
            let b = self.bottom.load(Ordering::Acquire);

            if (b.wrapping_sub(t) as isize) <= 0 {
                return None;
            }

            let task_idx = self.buffer[t & DEQUE_MASK].load(Ordering::Relaxed);
            if self
                .top
                .compare_exchange(t, t.wrapping_add(1), Ordering::SeqCst, Ordering::Relaxed)
                .is_ok()
            {
                Some(task_idx)
            } else {
                None
            }
        }

        /// Returns `true` if the deque is currently observed to be empty.
        #[inline]
        pub fn is_empty(&self) -> bool {
            let t = self.top.load(Ordering::Relaxed);
            let b = self.bottom.load(Ordering::Relaxed);
            (b.wrapping_sub(t) as isize) <= 0
        }

        /// Returns the approximate number of tasks currently queued.
        #[inline]
        pub fn len(&self) -> usize {
            let t = self.top.load(Ordering::Relaxed);
            let b = self.bottom.load(Ordering::Relaxed);
            let diff = b.wrapping_sub(t) as isize;
            if diff <= 0 {
                0
            } else if diff as usize > DEQUE_CAPACITY {
                DEQUE_CAPACITY
            } else {
                diff as usize
            }
        }
    }
}

pub mod percpu {
    use core::sync::atomic::{AtomicBool, Ordering};

    use super::deque::ChaseLevDeque;
    use crate::types::MAX_TASKS;

    pub const MAX_CPU_CORES: usize = 16;

    pub static CPU_RUNQUEUES: [ChaseLevDeque; MAX_CPU_CORES] =
        [const { ChaseLevDeque::new() }; MAX_CPU_CORES];

    pub static TASK_QUEUED: [AtomicBool; MAX_TASKS] = [const { AtomicBool::new(false) }; MAX_TASKS];

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
            let current_core = keira_arch::cpu::get_current_core_id();
            if target_core != current_core {
                keira_arch::interrupts::smp::smp_send_reschedule(target_core);
            }
            true
        } else {
            TASK_QUEUED[task_idx].store(false, Ordering::Release);
            false
        }
    }

    #[inline]
    pub fn push_local(core_id: usize, task_idx: usize) -> bool {
        enqueue_task(core_id, task_idx)
    }

    pub fn pop_local(core_id: usize) -> Option<usize> {
        let target_core = core_id % MAX_CPU_CORES;
        let task_idx = CPU_RUNQUEUES[target_core].pop()?;
        if task_idx < MAX_TASKS {
            TASK_QUEUED[task_idx].store(false, Ordering::Release);
        }
        Some(task_idx)
    }

    pub fn steal_from(victim_core: usize) -> Option<usize> {
        let target_core = victim_core % MAX_CPU_CORES;
        let task_idx = CPU_RUNQUEUES[target_core].steal()?;
        if task_idx < MAX_TASKS {
            TASK_QUEUED[task_idx].store(false, Ordering::Release);
        }
        Some(task_idx)
    }

    pub fn mark_task_dequeued(task_idx: usize) {
        if task_idx < MAX_TASKS {
            TASK_QUEUED[task_idx].store(false, Ordering::Release);
        }
    }

    pub fn is_task_queued(task_idx: usize) -> bool {
        if task_idx < MAX_TASKS {
            TASK_QUEUED[task_idx].load(Ordering::Acquire)
        } else {
            false
        }
    }

    pub fn runqueue_len(core_id: usize) -> usize {
        CPU_RUNQUEUES[core_id % MAX_CPU_CORES].len()
    }
}

pub use self::balancer::*;
pub use self::deque::*;
pub use self::percpu::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deque_lifo_local_push_pop() {
        let deque = ChaseLevDeque::new();
        assert!(deque.is_empty());
        assert_eq!(deque.len(), 0);

        assert!(deque.push(10).is_ok());
        assert!(deque.push(20).is_ok());
        assert!(deque.push(30).is_ok());
        assert_eq!(deque.len(), 3);
        assert!(!deque.is_empty());

        // Owner pops from bottom in LIFO order
        assert_eq!(deque.pop(), Some(30));
        assert_eq!(deque.pop(), Some(20));
        assert_eq!(deque.pop(), Some(10));
        assert_eq!(deque.pop(), None);
        assert!(deque.is_empty());
        assert_eq!(deque.len(), 0);
    }

    #[test]
    fn test_deque_fifo_steal() {
        let deque = ChaseLevDeque::new();
        assert!(deque.push(1).is_ok());
        assert!(deque.push(2).is_ok());
        assert!(deque.push(3).is_ok());

        // Remote thieves steal from top in FIFO order
        assert_eq!(deque.steal(), Some(1));
        assert_eq!(deque.steal(), Some(2));
        assert_eq!(deque.steal(), Some(3));
        assert_eq!(deque.steal(), None);
        assert!(deque.is_empty());
    }

    #[test]
    fn test_deque_capacity_bounds() {
        let deque = ChaseLevDeque::new();
        for i in 0..DEQUE_CAPACITY {
            assert!(deque.push(i).is_ok());
        }
        assert_eq!(deque.len(), DEQUE_CAPACITY);

        // Exceeding capacity must return Err with rejected element
        assert_eq!(deque.push(999), Err(999));

        // Pop one item, then push must succeed
        assert_eq!(deque.pop(), Some(DEQUE_CAPACITY - 1));
        assert!(deque.push(100).is_ok());
    }

    #[test]
    fn test_enqueue_deduplication() {
        let task_id = 42;
        // Task 0 must never be enqueued
        assert!(!enqueue_task(0, 0));

        // Valid task enqueuing
        assert!(enqueue_task(0, task_id));
        assert!(is_task_queued(task_id));

        // Duplicate enqueue attempts must be rejected
        assert!(!enqueue_task(0, task_id));
        assert!(!enqueue_task(1, task_id));

        // Pop releases the queued bit
        assert_eq!(pop_local(0), Some(task_id));
        assert!(!is_task_queued(task_id));

        // Can be enqueued again after being popped
        assert!(enqueue_task(1, task_id));
        assert_eq!(steal_from(1), Some(task_id));
        assert!(!is_task_queued(task_id));
    }

    #[test]
    fn test_empty_pop_and_steal() {
        let deque = ChaseLevDeque::new();
        assert_eq!(deque.pop(), None);
        assert_eq!(deque.steal(), None);
        assert!(deque.is_empty());
    }
}
