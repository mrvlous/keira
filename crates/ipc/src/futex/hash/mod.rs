// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! In-kernel futex hash table and syscall dispatchers.

pub mod table;

pub use table::{
    futex_reset, get_futex_stats, get_futex_table, sys_futex, FUTEX_TABLE, TOTAL_FUTEX_REQUEUES,
    TOTAL_FUTEX_WAITS, TOTAL_FUTEX_WAKES,
};
