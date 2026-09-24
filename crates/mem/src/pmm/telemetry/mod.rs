// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Memory usage telemetry, capacity reporting, and structural consistency validation.

pub mod invariants;
pub mod metrics;

pub use invariants::{verify_pmm_invariants, verify_pmm_invariants_locked};
pub use metrics::{
    free_memory, get_stats, max_physical_address, total_memory, total_usable_memory, used_memory,
};
