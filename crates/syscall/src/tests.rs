// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Integration tests for the system call subsystem.

#[cfg(test)]
mod tests {
    use crate::*;

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
    fn test_syscall_table_mapping_and_execution() {
        let ret = syscall_dispatcher(9999, 0, 0, 0, 0, 0, 0);
        assert_eq!(ret, errno_to_ret(ENOSYS));

        assert_eq!(table::syscall_name(SYS_PUTC), "sys_putc");
        assert_eq!(table::syscall_name(SYS_EXIT), "sys_exit");
        assert_eq!(table::syscall_name(SYS_OPEN), "sys_open");
        assert_eq!(table::syscall_name(SYS_FORK), "sys_fork");
    }

    #[test]
    fn test_tss_and_stack_configuration() {
        unsafe {
            let stack_addr = 0xFFFF_8000_2000_4000usize;
            set_kernel_stack(stack_addr);
            #[cfg(target_arch = "x86_64")]
            {
                let rsp0 = TSS.rsp0;
                assert_eq!(rsp0, stack_addr as u64);
            }
            #[cfg(target_arch = "x86")]
            {
                let esp0 = TSS.esp0;
                assert_eq!(esp0, stack_addr as u32);
            }
        }
    }

    #[test]
    fn test_user_pointer_validation_boundaries() {
        assert!(unsafe { validate_user_ptr(0, 0, false) }.is_ok());
        assert_eq!(unsafe { validate_user_ptr(0, 32, false) }, Err(EFAULT));
        assert_eq!(
            unsafe { validate_user_ptr(USER_MAX_ADDR, 32, false) },
            Err(EFAULT)
        );
    }

    #[test]
    fn test_errno_translation_and_ret() {
        assert_eq!(errno_to_ret(EPERM), (-1i64) as u64);
        assert_eq!(errno_to_ret(EBADF), (-9i64) as u64);
        assert_eq!(ret_to_errno((-9i64) as u64), Some(9));
    }

    #[test]
    fn test_exception_name_and_signals() {
        assert_eq!(exception::exception_name(0), "Division by Zero (#DE)");
        assert_eq!(
            exception::signal_name(keira_task::signal::SIGSEGV),
            "SIGSEGV"
        );
    }
}
