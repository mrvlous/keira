// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Unit tests for shell state, spinlock safety, and environment variables.

#[cfg(test)]
pub mod test {
    use crate::state::*;

    #[test]
    fn test_spin_lock_execution() {
        let val = with_spin_lock(|| 42);
        assert_eq!(val, 42);
    }

    #[test]
    fn test_get_and_set_env_var() {
        let mut buf = [0u8; 64];
        unsafe {
            let res = get_env_var("PATH", &mut buf);
            assert!(res.is_ok());

            let res_user = get_env_var("USER", &mut buf);
            assert!(res_user.is_ok());

            let set_res = set_env_var("PATH", "/custom/bin");
            assert!(set_res.is_ok());

            let get_res = get_env_var("PATH", &mut buf).unwrap();
            assert_eq!(&buf[..get_res], b"/custom/bin");

            // Reset back
            let _ = set_env_var("PATH", "/system/bin");
        }
    }
}
