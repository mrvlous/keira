// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Unit tests for system call dispatcher, parameter validation, and routing.

#[cfg(test)]
pub mod test {
    use crate::dispatcher::*;
    use crate::user_copy::errno::*;

    #[test]
    fn test_validate_fd_bounds() {
        for fd in 0..32 {
            assert!(validate_fd(fd).is_ok());
        }
        assert_eq!(validate_fd(-1), Err(EBADF));
        assert_eq!(validate_fd(32), Err(EBADF));
        assert_eq!(validate_fd(100), Err(EBADF));
    }

    #[test]
    fn test_sanitize_ret_behavior() {
        assert_eq!(sanitize_ret(1, 0xDEAD_BEEF), 0xDEAD_BEEF + 1);
        assert_eq!(sanitize_ret(2, 0xDEAD_BEEF), 0xDEAD_BEEF);
        assert_eq!(sanitize_ret(5, 0x1234), 0x1234);
    }

    #[test]
    fn test_unknown_syscall_returns_enosys() {
        let ret = syscall_dispatcher(9999, 0, 0, 0, 0, 0, 0);
        assert_eq!(ret, errno_to_ret(ENOSYS));
    }

    #[test]
    fn test_uptime_and_pid_syscalls() {
        let uptime = syscall_dispatcher(4, 0, 0, 0, 0, 0, 0);
        let pid = syscall_dispatcher(14, 0, 0, 0, 0, 0, 0);
        let _ = (uptime, pid);
    }

    #[test]
    fn test_cleanup_hook_execution() {
        syscall_task_cleanup_hook(0);
    }
}
