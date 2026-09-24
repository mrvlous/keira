// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Text buffer storage, disk synchronization, and status formatting.

pub mod file;
pub mod status;

pub use file::{editor_save_file, editor_start};
pub use status::{
    append_bytes, format_u64, get_file_last_line, get_total_lines, set_status_cur_pos,
    set_status_msg_read, set_status_msg_wrote,
};
