// SPDX-License-Identifier: GPL-2.0-only

//! Task State Segment (TSS) layout and kernel privilege stack management.

pub mod stack;
pub mod types;

pub use stack::{get_boot_kernel_stack, set_kernel_stack, BOOT_KERNEL_STACK_TOP, TSS};
pub use types::TaskStateSegment;
