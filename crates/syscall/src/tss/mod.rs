// SPDX-License-Identifier: GPL-2.0-only

//! Task State Segment (TSS), RSP0/ESP0 kernel privilege stack and userland transition setup.

pub mod privilege;
pub mod segment;

#[cfg(test)]
mod tests;

pub use privilege::init_user_mode;
pub use segment::{get_boot_kernel_stack, set_kernel_stack, TaskStateSegment, TSS};
