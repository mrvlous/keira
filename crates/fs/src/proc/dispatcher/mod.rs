// SPDX-License-Identifier: GPL-2.0-only

//! ProcFS node lookup, validation and read dispatcher.

pub mod read;

pub use self::read::{
    exists, get_pid_dir_entry, get_proc_entry, is_proc_dir, read_proc_file, PID_DIR_ENTRIES,
    PROC_ENTRIES,
};
