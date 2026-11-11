// SPDX-License-Identifier: GPL-2.0-only

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
