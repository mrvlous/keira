// SPDX-License-Identifier: GPL-2.0-only

//! Synchronization guards, CPU affinity and reentrancy detection for PMM.

pub mod guard;

pub use guard::{get_current_cpu_id, PmmGuard};

#[cfg(test)]
pub use guard::{clear_test_cpu_id, set_test_cpu_id, TEST_MUTEX};
