// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Hardware breakpoint and memory watchpoint control subsystem.

pub mod hw;

pub use hw::{
    check_and_clear_status, clear_watchpoint, read_dr0, read_dr1, read_dr2, read_dr3, read_dr6,
    read_dr7, set_watchpoint, write_dr0, write_dr1, write_dr2, write_dr3, write_dr6, write_dr7,
    WatchpointCondition, WatchpointEntry, WatchpointSize, WATCHPOINTS,
};
