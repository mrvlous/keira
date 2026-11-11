// SPDX-License-Identifier: GPL-2.0-only

//! Resource quota management and memory limits.

pub mod limits;
pub mod memory;

pub use limits::set_cgroup_limits;
pub use memory::check_memory_limit;
