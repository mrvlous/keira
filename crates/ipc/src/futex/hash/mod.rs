// SPDX-License-Identifier: GPL-2.0-only

//! In-kernel futex hash table and syscall dispatchers.

pub mod table;

pub use table::{
    futex_reset, get_futex_stats, get_futex_table, sys_futex, FUTEX_TABLE, TOTAL_FUTEX_REQUEUES,
    TOTAL_FUTEX_WAITS, TOTAL_FUTEX_WAKES,
};
