// SPDX-License-Identifier: GPL-2.0-only

//! Address Space Layout Randomization (ASLR) and virtual address protection.

pub mod engine;

pub use engine::{
    generate_task_aslr_offsets, get_aslr_limits, get_aslr_policy, get_aslr_status, set_aslr_policy,
    AslrStatus, ASLR_MODE_CONSERVATIVE, ASLR_MODE_DISABLED, ASLR_MODE_FULL,
};
