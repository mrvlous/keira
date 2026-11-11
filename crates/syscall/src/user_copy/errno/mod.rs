// SPDX-License-Identifier: GPL-2.0-only

//! Standard POSIX error codes and conversion utilities for system call returns.

pub mod codes;
pub mod conv;

pub use codes::*;
pub use conv::{errno_to_ret, ret_to_errno};
