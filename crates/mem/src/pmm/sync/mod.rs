// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Synchronization guards, CPU affinity, and reentrancy detection for PMM.

pub mod guard;

pub use guard::{get_current_cpu_id, PmmGuard};

#[cfg(test)]
pub use guard::{clear_test_cpu_id, set_test_cpu_id, TEST_MUTEX};
