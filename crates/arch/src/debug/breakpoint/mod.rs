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

//! Hardware breakpoint and memory watchpoint control subsystem.

pub mod hw;

pub use hw::{
    check_and_clear_status, clear_watchpoint, read_dr0, read_dr1, read_dr2, read_dr3, read_dr6,
    read_dr7, set_watchpoint, write_dr0, write_dr1, write_dr2, write_dr3, write_dr6, write_dr7,
    WatchpointCondition, WatchpointEntry, WatchpointSize, WATCHPOINTS,
};
