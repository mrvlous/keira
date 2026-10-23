// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Unit tests for system call table numbering and vector categorization.

#[cfg(test)]
pub mod test {
    use crate::table::*;

    #[test]
    fn test_syscall_number_constants() {
        assert_eq!(SYS_PUTC, 1);
        assert_eq!(SYS_EXIT, 2);
        assert_eq!(SYS_OPEN, 6);
        assert_eq!(SYS_READ, 7);
        assert_eq!(SYS_WRITE, 8);
        assert_eq!(SYS_MMAP, 20);
        assert_eq!(SYS_FORK, 30);
        assert_eq!(SYS_DUP2, 85);
        assert_eq!(SYS_GETDENTS, 86);
        assert_eq!(SYS_UNLINK, 87);
        assert_eq!(SYS_MKDIR, 88);
        assert_eq!(SYS_PING, 89);
        assert_eq!(SYS_GETEUID, 90);
        assert_eq!(SYS_GETEGID, 91);
        assert_eq!(SYS_SYMLINK, 92);
        assert_eq!(SYS_READLINK, 93);
        assert_eq!(SYS_CHMOD, 97);
        assert_eq!(SYS_CHOWN, 98);
        assert_eq!(SYS_STAT, 99);
    }

    #[test]
    fn test_syscall_name_resolutions() {
        assert_eq!(syscall_name(SYS_PUTC), "sys_putc");
        assert_eq!(syscall_name(SYS_EXIT), "sys_exit");
        assert_eq!(syscall_name(SYS_OPEN), "sys_open");
        assert_eq!(syscall_name(SYS_MMAP), "sys_mmap");
        assert_eq!(syscall_name(SYS_FORK), "sys_fork");
        assert_eq!(syscall_name(SYS_DUP2), "sys_dup2");
        assert_eq!(syscall_name(SYS_GETDENTS), "sys_getdents");
        assert_eq!(syscall_name(SYS_UNLINK), "sys_unlink");
        assert_eq!(syscall_name(SYS_MKDIR), "sys_mkdir");
        assert_eq!(syscall_name(SYS_PING), "sys_ping");
        assert_eq!(syscall_name(SYS_GETEUID), "sys_geteuid");
        assert_eq!(syscall_name(SYS_GETEGID), "sys_getegid");
        assert_eq!(syscall_name(SYS_SYMLINK), "sys_symlink");
        assert_eq!(syscall_name(SYS_READLINK), "sys_readlink");
        assert_eq!(syscall_name(SYS_CHMOD), "sys_chmod");
        assert_eq!(syscall_name(SYS_CHOWN), "sys_chown");
        assert_eq!(syscall_name(SYS_STAT), "sys_stat");
        assert_eq!(syscall_name(999), "sys_unknown");
    }

    #[test]
    fn test_syscall_categories() {
        assert!(is_valid_syscall(SYS_OPEN));
        assert!(is_valid_syscall(SYS_MKDIR));
        assert!(is_valid_syscall(SYS_SYMLINK));
        assert!(is_valid_syscall(SYS_READLINK));
        assert!(is_valid_syscall(SYS_CHMOD));
        assert!(is_valid_syscall(SYS_CHOWN));
        assert!(is_valid_syscall(SYS_STAT));
        assert!(!is_valid_syscall(999));

        assert!(is_io_syscall(SYS_OPEN));
        assert!(is_io_syscall(SYS_WRITE));
        assert!(is_io_syscall(SYS_MKDIR));
        assert!(is_io_syscall(SYS_SYMLINK));
        assert!(is_io_syscall(SYS_READLINK));
        assert!(is_io_syscall(SYS_CHMOD));
        assert!(is_io_syscall(SYS_CHOWN));
        assert!(is_io_syscall(SYS_STAT));
        assert!(!is_io_syscall(SYS_MMAP));

        assert!(is_memory_syscall(SYS_MMAP));
        assert!(is_memory_syscall(SYS_BRK));
        assert!(!is_memory_syscall(SYS_FORK));

        assert!(is_process_syscall(SYS_FORK));
        assert!(is_process_syscall(SYS_EXIT));
        assert!(!is_process_syscall(SYS_READ));

        assert!(is_ipc_syscall(SYS_PIPE));
        assert!(is_ipc_syscall(SYS_FUTEX));
        assert!(!is_ipc_syscall(SYS_OPEN));
    }
}
