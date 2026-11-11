// SPDX-License-Identifier: GPL-2.0-only

//! Task lifecycle transitions, termination and process synchronization.

pub mod exit;
pub mod reap;
pub mod wait;

pub use exit::{exit_current, stop_task};
pub use reap::{reap_orphaned_zombies, reap_orphaned_zombies_locked};
pub use wait::{sys_waitpid, wait_for_task};
