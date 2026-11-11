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

//! File descriptor validation routines.

use keira_task::types::MAX_FDS;

use crate::user_copy::errno::EBADF;

/// Validates whether a file descriptor index falls within the valid range `[0, MAX_FDS)`.
pub fn validate_fd(fd: i32) -> Result<(), i64> {
    if (0..MAX_FDS as i32).contains(&fd) {
        Ok(())
    } else {
        Err(EBADF)
    }
}
