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

//! Text buffer storage, disk synchronization and status formatting.

pub mod file;
pub mod status;

pub use file::{editor_save_file, editor_start};
pub use status::{
    append_bytes, format_u64, get_file_last_line, get_total_lines, set_status_cur_pos,
    set_status_msg_read, set_status_msg_wrote,
};
