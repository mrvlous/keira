// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Chase-Lev lock-free circular work-stealing deque.
//!
//! Implements a single-producer multi-consumer work-stealing deque based on the
//! Chase-Lev algorithm. The owning CPU pushes and pops tasks from the bottom,
//! while idle remote processors concurrently steal tasks from the top using
//! atomic Compare-And-Swap (CAS), protected by Epoch-Based Reclamation (EBR).

use core::sync::atomic::{fence, AtomicUsize, Ordering};

/// Fixed capacity of the lock-free circular deque (must be a power of two).
pub const DEQUE_CAPACITY: usize = 64;

/// Bitmask for circular array indexing into the deque.
pub const DEQUE_MASK: usize = DEQUE_CAPACITY - 1;

/// Lock-free single-producer multi-consumer Chase-Lev work-stealing deque.
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
