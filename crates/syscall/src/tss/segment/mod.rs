// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Task State Segment (TSS) layout and kernel privilege stack management.

pub mod stack;
pub mod types;

pub use stack::{get_boot_kernel_stack, set_kernel_stack, BOOT_KERNEL_STACK_TOP, TSS};
pub use types::TaskStateSegment;
