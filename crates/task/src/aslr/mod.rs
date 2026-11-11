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

//! Address Space Layout Randomization (ASLR) and virtual address protection.

pub mod engine;

pub use engine::{
    generate_task_aslr_offsets, get_aslr_limits, get_aslr_policy, get_aslr_status, set_aslr_policy,
    AslrStatus, ASLR_MODE_CONSERVATIVE, ASLR_MODE_DISABLED, ASLR_MODE_FULL,
};
