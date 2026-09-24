// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Task State Segment (TSS), RSP0/ESP0 kernel privilege stack, and userland transition setup.

pub mod privilege;
pub mod segment;

#[cfg(test)]
mod tests;

pub use privilege::init_user_mode;
pub use segment::{get_boot_kernel_stack, set_kernel_stack, TaskStateSegment, TSS};
