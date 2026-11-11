// SPDX-License-Identifier: GPL-2.0-only
/*
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra <mrvlous@proton.me>
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 */

//! Task State Segment (TSS), RSP0/ESP0 kernel privilege stack and userland transition setup.

pub mod privilege;
pub mod segment;

#[cfg(test)]
mod tests;

pub use privilege::init_user_mode;
pub use segment::{get_boot_kernel_stack, set_kernel_stack, TaskStateSegment, TSS};
