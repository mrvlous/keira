// SPDX-License-Identifier: GPL-2.0-only

//! Memory usage telemetry, capacity reporting and structural consistency validation.

pub mod invariants;
pub mod metrics;

pub use invariants::{verify_pmm_invariants, verify_pmm_invariants_locked};
pub use metrics::{
    free_memory, get_stats, max_physical_address, total_memory, total_usable_memory, used_memory,
};
