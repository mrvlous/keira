// SPDX-License-Identifier: GPL-2.0-only

//! POSIX signal mask manipulation commands (`sigprocmask`).

pub const SIG_BLOCK: i32 = 0;
pub const SIG_UNBLOCK: i32 = 1;
pub const SIG_SETMASK: i32 = 2;
