// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Unit tests for user space pointer validation, copying, and errno conversions.

#[cfg(test)]
pub mod test {
    use crate::user_copy::*;

    #[test]
    fn test_errno_conversions() {
        assert_eq!(errno_to_ret(EPERM), (-1i64) as u64);
        assert_eq!(errno_to_ret(EINVAL), (-22i64) as u64);
        assert_eq!(errno_to_ret(ENOSYS), (-38i64) as u64);

        assert_eq!(ret_to_errno((-1i64) as u64), Some(1));
        assert_eq!(ret_to_errno((-22i64) as u64), Some(22));
        assert_eq!(ret_to_errno(0), None);
        assert_eq!(ret_to_errno(1234), None);
    }

    #[test]
    fn test_validate_user_ptr_ranges() {
        // Zero length is always allowed
        assert!(unsafe { validate_user_ptr(0, 0, false) }.is_ok());
        assert!(unsafe { validate_user_ptr(0x5000, 0, true) }.is_ok());

        // Below minimum user address
        assert_eq!(unsafe { validate_user_ptr(0, 64, false) }, Err(EFAULT));
        assert_eq!(unsafe { validate_user_ptr(0x8000, 64, true) }, Err(EFAULT));

        // Within user address bounds
        assert!(unsafe { validate_user_ptr(USER_MIN_ADDR, 0x1000, false) }.is_ok());
        assert!(unsafe { validate_user_ptr(0x20000, 64, true) }.is_ok());

        // Above maximum user address
        assert_eq!(
            unsafe { validate_user_ptr(USER_MAX_ADDR, 0x1000, false) },
            Err(EFAULT)
        );

        // Arithmetic overflow
        assert_eq!(
            unsafe { validate_user_ptr(u64::MAX - 10, 100, false) },
            Err(EFAULT)
        );
    }

    #[test]
    fn test_read_user_string_validation() {
        let mut buf = [0u8; 32];

        // Null pointer check
        assert_eq!(
            unsafe { read_user_string(core::ptr::null(), &mut buf) },
            Err(EFAULT)
        );

        // Pointer below USER_MIN_ADDR
        let invalid_ptr = 0x100 as *const u8;
        assert_eq!(
            unsafe { read_user_string(invalid_ptr, &mut buf) },
            Err(EFAULT)
        );
    }
}
