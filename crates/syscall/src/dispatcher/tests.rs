// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Unit tests for system call dispatcher, parameter validation and routing.

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

    #[test]
    fn test_chdir_validation() {
        let ret = syscall_dispatcher(16, 0, 0, 0, 0, 0, 0);
        assert_eq!(ret, errno_to_ret(EFAULT));
    }

    #[test]
    fn test_normalize_path_resolution() {
        let mut buf = [0u8; 64];
        let len =
            crate::dispatcher::handlers::process::normalize_path("/", "dev", &mut buf).unwrap();
        assert_eq!(&buf[..len], b"/dev");

        let len =
            crate::dispatcher::handlers::process::normalize_path("/dev", ".", &mut buf).unwrap();
        assert_eq!(&buf[..len], b"/dev");

        let len =
            crate::dispatcher::handlers::process::normalize_path("/dev", "null", &mut buf).unwrap();
        assert_eq!(&buf[..len], b"/dev/null");

        let len =
            crate::dispatcher::handlers::process::normalize_path("/dev", "..", &mut buf).unwrap();
        assert_eq!(&buf[..len], b"/");

        let len = crate::dispatcher::handlers::process::normalize_path(
            "/dev",
            "../etc/hostname",
            &mut buf,
        )
        .unwrap();
        assert_eq!(&buf[..len], b"/etc/hostname");
    }

    #[test]
    fn test_symlink_and_readlink_dispatch() {
        // Calling symlink with null pointer should return EFAULT
        let ret = syscall_dispatcher(92, 0, 0, 0, 0, 0, 0);
        assert_eq!(ret, errno_to_ret(EFAULT));

        // Calling readlink with null pointer should return EFAULT
        let ret = syscall_dispatcher(93, 0, 0, 10, 0, 0, 0);
        assert_eq!(ret, errno_to_ret(EFAULT));
    }

    #[test]
    fn test_mount_and_umount_dispatch() {
        // Calling mount with null pointer should return EINVAL
        let ret = syscall_dispatcher(94, 0, 0, 0, 0, 0, 0);
        assert_eq!(ret, errno_to_ret(EINVAL));

        // Calling umount with null pointer should return EINVAL
        let ret = syscall_dispatcher(95, 0, 0, 0, 0, 0, 0);
        assert_eq!(ret, errno_to_ret(EINVAL));
    }
}
