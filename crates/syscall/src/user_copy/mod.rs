// SPDX-License-Identifier: GPL-2.0-only

//! Centralized safe user space memory validation, range checking and copying primitives.

pub mod copy;
pub mod errno;
pub mod string;
pub mod validate;

#[cfg(test)]
mod tests;

pub use copy::{copy_from_user, copy_to_user, copy_val_from_user, copy_val_to_user};
pub use errno::{
    errno_to_ret, ret_to_errno, EACCES, EAFNOSUPPORT, EAGAIN, EBADF, EBUSY, ECHILD, EEXIST, EFAULT,
    EFBIG, EINTR, EINVAL, EIO, EISDIR, ELOOP, EMFILE, ENAMETOOLONG, ENFILE, ENODEV, ENOENT, ENOMEM,
    ENOSPC, ENOSYS, ENOTDIR, ENOTEMPTY, EPERM, EPIPE, EPROTONOSUPPORT, ESRCH,
};
pub use string::{read_user_string, read_user_string_bounded};
pub use validate::{validate_user_ptr, USER_MAX_ADDR, USER_MIN_ADDR};
