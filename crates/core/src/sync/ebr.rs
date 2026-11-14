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

//! Epoch-Based Reclamation (EBR) synchronization framework.

pub mod epoch {
    use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

    pub const MAX_EBR_PARTICIPANTS: usize = 16;

    pub static GLOBAL_EPOCH: AtomicUsize = AtomicUsize::new(0);

    #[derive(Debug)]
    pub struct Participant {
        active: AtomicBool,
        epoch: AtomicUsize,
    }

    impl Default for Participant {
        fn default() -> Self {
            Self::new()
        }
    }

    impl Participant {
        /// Creates a new unpinned participant instance.
        pub const fn new() -> Self {
            Self {
                active: AtomicBool::new(false),
                epoch: AtomicUsize::new(0),
            }
        }

        /// Marks the participant as active in the current global epoch.
        #[inline]
        pub fn pin(&self, current_epoch: usize) {
            self.epoch.store(current_epoch, Ordering::Release);
            self.active.store(true, Ordering::Release);
        }

        /// Marks the participant as unpinned and inactive.
        #[inline]
        pub fn unpin(&self) {
            self.active.store(false, Ordering::Release);
        }

        /// Returns `true` if the participant is currently pinned.
        #[inline]
        pub fn is_pinned(&self) -> bool {
            self.active.load(Ordering::Acquire)
        }

        /// Returns the epoch observed when this participant was pinned.
        #[inline]
        pub fn pinned_epoch(&self) -> usize {
            self.epoch.load(Ordering::Acquire)
        }
    }

    pub static PARTICIPANTS: [Participant; MAX_EBR_PARTICIPANTS] = [
        Participant::new(),
        Participant::new(),
        Participant::new(),
        Participant::new(),
        Participant::new(),
        Participant::new(),
        Participant::new(),
        Participant::new(),
        Participant::new(),
        Participant::new(),
        Participant::new(),
        Participant::new(),
        Participant::new(),
        Participant::new(),
        Participant::new(),
        Participant::new(),
    ];

    #[inline]
    pub fn current_epoch() -> usize {
        GLOBAL_EPOCH.load(Ordering::Acquire)
    }

    pub fn try_advance_epoch() -> bool {
        let curr = GLOBAL_EPOCH.load(Ordering::Acquire);

        // If any active participant is pinned at an older epoch, epoch cannot advance yet
        for participant in &PARTICIPANTS {
            if participant.is_pinned() && participant.pinned_epoch() < curr {
                return false;
            }
        }

        GLOBAL_EPOCH
            .compare_exchange(
                curr,
                curr.wrapping_add(1),
                Ordering::AcqRel,
                Ordering::Relaxed,
            )
            .is_ok()
    }
}

pub use self::epoch::{current_epoch, try_advance_epoch, MAX_EBR_PARTICIPANTS, PARTICIPANTS};

pub mod bag {
    use crate::sync::ebr::epoch::{current_epoch, try_advance_epoch};
    use crate::sync::spinlock::IrqSpinLock;

    pub const MAX_BAG_CAPACITY: usize = 64;

    #[derive(Clone, Copy, Debug)]
    pub struct RetiredEntry {
        /// Opaque pointer or address representing the retired resource.
        pub ptr: usize,
        /// The global epoch at which this resource was retired.
        pub epoch: usize,
        /// Optional callback invoked when the resource is safe to deallocate.
        pub reclaim: Option<fn(usize)>,
    }

    impl RetiredEntry {
        /// Creates an empty invalid entry.
        pub const fn empty() -> Self {
            Self {
                ptr: 0,
                epoch: 0,
                reclaim: None,
            }
        }
    }

    pub struct GarbageBag {
        lock: IrqSpinLock,
        entries: [RetiredEntry; MAX_BAG_CAPACITY],
        count: usize,
    }

    impl Default for GarbageBag {
        fn default() -> Self {
            Self::new()
        }
    }

    impl GarbageBag {
        /// Constructs a new empty `GarbageBag`.
        pub const fn new() -> Self {
            Self {
                lock: IrqSpinLock::new(),
                entries: [RetiredEntry::empty(); MAX_BAG_CAPACITY],
                count: 0,
            }
        }

        /// Retires an address or pointer under the current global epoch.
        ///
        /// The entry will be retained until all active participants have advanced
        /// past this epoch.
        pub fn retire(
            &mut self,
            ptr: usize,
            reclaim: Option<fn(usize)>,
        ) -> Result<(), &'static str> {
            let _guard = self.lock.lock();
            if self.count >= MAX_BAG_CAPACITY {
                return Err("GarbageBag capacity exceeded");
            }

            let curr_epoch = current_epoch();
            self.entries[self.count] = RetiredEntry {
                ptr,
                epoch: curr_epoch,
                reclaim,
            };
            self.count += 1;
            Ok(())
        }

        /// Reclaims all retired entries that are strictly older than the current epoch by at least 2.
        ///
        /// In 3-epoch EBR, an object retired in epoch `e` is safe to free when the global epoch
        /// has reached at least `e + 2`, guaranteeing that no participant is still reading in epoch `e`.
        ///
        /// Returns the number of items successfully reclaimed.
        pub fn collect(&mut self) -> usize {
            let _guard = self.lock.lock();
            let curr = current_epoch();
            let mut reclaimed_count = 0;
            let mut new_count = 0;

            for i in 0..self.count {
                let entry = self.entries[i];
                // Safe to reclaim if epoch difference is at least 2
                if curr >= entry.epoch.saturating_add(2) {
                    if let Some(drop_fn) = entry.reclaim {
                        drop_fn(entry.ptr);
                    }
                    reclaimed_count += 1;
                } else {
                    self.entries[new_count] = entry;
                    new_count += 1;
                }
            }

            self.count = new_count;
            reclaimed_count
        }

        /// Attempts to advance the global epoch and subsequently collects eligible retired entries.
        ///
        /// Returns `(epoch_advanced, reclaimed_count)`.
        pub fn advance_and_collect(&mut self) -> (bool, usize) {
            let advanced = try_advance_epoch();
            let reclaimed = self.collect();
            (advanced, reclaimed)
        }

        /// Returns the count of pending retired items currently stored in the bag.
        pub fn pending_count(&self) -> usize {
            let _guard = self.lock.lock();
            self.count
        }
    }
}

pub use self::bag::{GarbageBag, RetiredEntry, MAX_BAG_CAPACITY};

pub mod guard {
    use crate::sync::ebr::epoch::{current_epoch, MAX_EBR_PARTICIPANTS, PARTICIPANTS};
    use crate::sync::spinlock::current_core_id;

    #[derive(Debug)]
    pub struct EpochGuard {
        participant_id: usize,
        epoch: usize,
    }

    impl EpochGuard {
        /// Returns the epoch observed when this guard was acquired.
        #[inline]
        pub fn epoch(&self) -> usize {
            self.epoch
        }

        /// Returns the participant index associated with this guard.
        #[inline]
        pub fn participant_id(&self) -> usize {
            self.participant_id
        }
    }

    impl Drop for EpochGuard {
        #[inline]
        fn drop(&mut self) {
            if self.participant_id < MAX_EBR_PARTICIPANTS {
                PARTICIPANTS[self.participant_id].unpin();
            }
        }
    }

    #[inline]
    pub fn pin() -> EpochGuard {
        let raw_id = current_core_id();
        let core_id = if raw_id >= 0 {
            (raw_id as usize) % MAX_EBR_PARTICIPANTS
        } else {
            0
        };
        let curr_epoch = current_epoch();
        PARTICIPANTS[core_id].pin(curr_epoch);
        EpochGuard {
            participant_id: core_id,
            epoch: curr_epoch,
        }
    }

    #[inline]
    pub fn pin_participant(id: usize) -> EpochGuard {
        let safe_id = id % MAX_EBR_PARTICIPANTS;
        let curr_epoch = current_epoch();
        PARTICIPANTS[safe_id].pin(curr_epoch);
        EpochGuard {
            participant_id: safe_id,
            epoch: curr_epoch,
        }
    }
}

pub use self::guard::{pin, pin_participant, EpochGuard};

#[cfg(test)]
pub mod tests {
    use super::*;
    use core::sync::atomic::{AtomicUsize, Ordering};

    static RECLAIM_COUNTER: AtomicUsize = AtomicUsize::new(0);
    static TEST_LOCK: crate::sync::SpinLock = crate::sync::SpinLock::new();

    struct TestGuard;
    impl Drop for TestGuard {
        fn drop(&mut self) {
            TEST_LOCK.unlock();
        }
    }

    fn lock_test() -> TestGuard {
        TEST_LOCK.lock();
        TestGuard
    }

    fn mock_reclaim(_val: usize) {
        RECLAIM_COUNTER.fetch_add(1, Ordering::SeqCst);
    }

    #[test]
    fn test_ebr_pin_unpin_lifecycle() {
        let _lock = lock_test();
        let initial_epoch = current_epoch();
        {
            let guard = pin_participant(0);
            assert_eq!(guard.epoch(), initial_epoch);
            assert!(PARTICIPANTS[0].is_pinned());
        }
        assert!(!PARTICIPANTS[0].is_pinned());
    }

    #[test]
    fn test_ebr_epoch_advancement_prevented_by_pinned_lagging_reader() {
        let _lock = lock_test();
        let start_epoch = current_epoch();
        let guard = pin_participant(1);
        assert_eq!(guard.epoch(), start_epoch);

        // If participant 1 is pinned at start_epoch, try_advance_epoch() can advance once to start_epoch + 1
        assert!(try_advance_epoch());
        assert_eq!(current_epoch(), start_epoch + 1);

        // Now participant 1 is pinned at start_epoch, which is < current_epoch (start_epoch + 1).
        // Advancement must now be prevented because participant 1 has not caught up!
        assert!(!try_advance_epoch());
        assert_eq!(current_epoch(), start_epoch + 1);

        // Drop the guard, unpinning participant 1
        drop(guard);
        assert!(!PARTICIPANTS[1].is_pinned());

        // Now advancement must succeed
        assert!(try_advance_epoch());
        assert_eq!(current_epoch(), start_epoch + 2);
    }

    #[test]
    fn test_ebr_garbage_bag_lifecycle() {
        let _lock = lock_test();
        let mut bag = GarbageBag::new();
        RECLAIM_COUNTER.store(0, Ordering::SeqCst);

        let start_epoch = current_epoch();
        assert!(bag.retire(0x1000, Some(mock_reclaim)).is_ok());
        assert_eq!(bag.pending_count(), 1);

        // Immediate collection in same epoch should not reclaim yet (must be >= epoch + 2)
        assert_eq!(bag.collect(), 0);
        assert_eq!(RECLAIM_COUNTER.load(Ordering::SeqCst), 0);
        assert_eq!(bag.pending_count(), 1);

        // Advance 1 epoch
        assert!(try_advance_epoch());
        assert_eq!(current_epoch(), start_epoch + 1);
        assert_eq!(bag.collect(), 0);
        assert_eq!(RECLAIM_COUNTER.load(Ordering::SeqCst), 0);

        // Advance 2nd epoch
        assert!(try_advance_epoch());
        assert_eq!(current_epoch(), start_epoch + 2);

        // Now collection should reclaim the retired item
        assert_eq!(bag.collect(), 1);
        assert_eq!(RECLAIM_COUNTER.load(Ordering::SeqCst), 1);
        assert_eq!(bag.pending_count(), 0);
    }
}
