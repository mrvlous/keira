// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Fixed-size generic circular ring buffer without heap allocation.

/// A fixed-capacity ring buffer suitable for queues and log streams.
pub struct RingBuffer<T: Copy, const CAP: usize> {
    buffer: [T; CAP],
    head: usize,
    tail: usize,
    count: usize,
}

impl<T: Copy, const CAP: usize> RingBuffer<T, CAP> {
    /// Create a new empty RingBuffer initialized with the specified filler element.
    pub const fn new(init_val: T) -> Self {
        Self {
            buffer: [init_val; CAP],
            head: 0,
            tail: 0,
            count: 0,
        }
    }

    /// Push an item to the ring buffer. If full, the oldest item is overwritten.
    pub fn push(&mut self, item: T) {
        self.buffer[self.tail] = item;
        self.tail = (self.tail + 1) % CAP;
        if self.count < CAP {
            self.count += 1;
        } else {
            self.head = (self.head + 1) % CAP;
        }
    }

    /// Pop the oldest item from the ring buffer.
    pub fn pop(&mut self) -> Option<T> {
        if self.count == 0 {
            None
        } else {
            let item = self.buffer[self.head];
            self.head = (self.head + 1) % CAP;
            self.count -= 1;
            Some(item)
        }
    }

    /// Return the current number of elements stored.
    pub const fn len(&self) -> usize {
        self.count
    }

    /// Check if the ring buffer is empty.
    pub const fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Check if the ring buffer is full.
    pub const fn is_full(&self) -> bool {
        self.count == CAP
    }

    /// Clear all elements from the ring buffer.
    pub fn clear(&mut self) {
        self.head = 0;
        self.tail = 0;
        self.count = 0;
    }
}
