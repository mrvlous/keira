// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! RAII guard representing a critical read section under Epoch-Based Reclamation.

use super::epoch::{current_epoch, MAX_EBR_PARTICIPANTS, PARTICIPANTS};
use crate::sync::spinlock::current_core_id;

/// RAII epoch guard ensuring the participant remains pinned during read operations.
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

/// Pins the current CPU core to the current global epoch, returning an RAII `EpochGuard`.
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

/// Pins a specific participant ID (useful for tests or custom thread assignments).
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
