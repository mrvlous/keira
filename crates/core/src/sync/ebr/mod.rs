// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Type-Safe Epoch-Based Reclamation (EBR) Subsystem.
//!
//! Provides lock-free, zero-contention, safe memory reclamation across multicore
//! SMP CPUs with compile-time safety guarantees, eliminating read-side locks
//! and protecting concurrently accessed data structures from premature destruction.

pub mod bag;
pub mod epoch;
pub mod guard;

#[cfg(test)]
mod tests;

pub use bag::{GarbageBag, RetiredEntry, MAX_BAG_CAPACITY};
pub use epoch::{
    current_epoch, try_advance_epoch, Participant, GLOBAL_EPOCH, MAX_EBR_PARTICIPANTS, PARTICIPANTS,
};
pub use guard::{pin, pin_participant, EpochGuard};
