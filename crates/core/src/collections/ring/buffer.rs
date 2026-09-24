// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Fixed-size circular ring buffer without heap allocation.
//!
//! Provides a bounded FIFO queue and circular logging stream for device drivers,
//! interrupt queues, serial FIFO buffers, and scheduler runqueues.

/// Fixed-capacity circular ring buffer.
///
/// Implements O(1) enqueue and dequeue operations without heap allocation.
/// When the buffer reaches capacity, subsequent pushes overwrite the oldest
/// unconsumed element, advancing the head pointer.
pub struct RingBuffer<T: Copy, const CAP: usize> {
    /// Underlying static storage array.
    buffer: [T; CAP],
    /// Index pointing to the oldest unconsumed element (dequeue position).
    head: usize,
    /// Index pointing to the next available write slot (enqueue position).
    tail: usize,
    /// Current number of elements stored within the ring buffer.
    count: usize,
}

impl<T: Copy, const CAP: usize> RingBuffer<T, CAP> {
    /// Constructs a new empty `RingBuffer` populated with the specified filler template.
    ///
    /// # Arguments
    ///
    /// * `init_val` - Value used to initialize underlying static array slots.
    pub const fn new(init_val: T) -> Self {
        Self {
            buffer: [init_val; CAP],
            head: 0,
            tail: 0,
            count: 0,
        }
    }

    /// Enqueues an item into the ring buffer.
    ///
    /// If the buffer is full, the oldest unread element is overwritten and
    /// the head pointer advances forward by one position.
    pub fn push(&mut self, item: T) {
        self.buffer[self.tail] = item;
        self.tail = (self.tail + 1) % CAP;
        if self.count < CAP {
            self.count += 1;
        } else {
            self.head = (self.head + 1) % CAP;
        }
    }

    /// Dequeues and returns the oldest item stored in the ring buffer.
    ///
    /// # Returns
    ///
    /// * `Some(T)` - If the buffer contains at least one unconsumed element.
    /// * `None` - If the buffer is empty.
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

    /// Peeks at the oldest element in the buffer without removing it.
    pub fn peek(&self) -> Option<T> {
        if self.count == 0 {
            None
        } else {
            Some(self.buffer[self.head])
        }
    }

    /// Returns the current number of elements currently stored.
    #[inline(always)]
    pub const fn len(&self) -> usize {
        self.count
    }

    /// Returns the maximum capacity of the ring buffer.
    #[inline(always)]
    pub const fn capacity(&self) -> usize {
        CAP
    }

    /// Returns true if the ring buffer contains no stored elements.
    #[inline(always)]
    pub const fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Returns true if the ring buffer has reached its maximum capacity.
    #[inline(always)]
    pub const fn is_full(&self) -> bool {
        self.count == CAP
    }

    /// Clears all elements from the ring buffer, resetting head and tail indices.
    pub fn clear(&mut self) {
        self.head = 0;
        self.tail = 0;
        self.count = 0;
    }
}
