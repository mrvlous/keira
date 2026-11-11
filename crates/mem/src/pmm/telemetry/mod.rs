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

//! Memory usage telemetry, capacity reporting and structural consistency validation.

pub mod invariants;
pub mod metrics;

pub use invariants::{verify_pmm_invariants, verify_pmm_invariants_locked};
pub use metrics::{
    free_memory, get_stats, max_physical_address, total_memory, total_usable_memory, used_memory,
};
