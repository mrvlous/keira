// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! ProcFS node lookup, validation and read dispatcher.

pub mod read;

pub use self::read::{
    exists, get_pid_dir_entry, get_proc_entry, is_proc_dir, read_proc_file, PID_DIR_ENTRIES,
    PROC_ENTRIES,
};
