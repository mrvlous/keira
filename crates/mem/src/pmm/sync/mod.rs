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

//! Synchronization guards, CPU affinity and reentrancy detection for PMM.

pub mod guard;

pub use guard::{get_current_cpu_id, PmmGuard};

#[cfg(test)]
pub use guard::{clear_test_cpu_id, set_test_cpu_id, TEST_MUTEX};
