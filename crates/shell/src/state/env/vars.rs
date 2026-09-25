// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Shell environment variables storage and lookup table.

pub static mut ENV_PATH: [u8; 64] = [
    b'/', b's', b'y', b's', b't', b'e', b'm', b'/', b'b', b'i', b'n', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];
pub static mut ENV_PATH_LEN: usize = 11;
pub static mut ENV_SHELL: [u8; 32] = *b"/system/bin/keira               ";
pub static mut ENV_SHELL_LEN: usize = 17;

/// Retrieves the string value of an environment variable.
pub unsafe fn get_env_var(name: &str, buf: &mut [u8]) -> Result<usize, &'static str> {
    match name {
        "PATH" => {
            let len = ENV_PATH_LEN;
            if buf.len() < len {
                return Err("Buffer too small");
            }
            buf[..len].copy_from_slice(&ENV_PATH[..len]);
            Ok(len)
        }
        "SHELL" => {
            let len = ENV_SHELL_LEN;
            if buf.len() < len {
                return Err("Buffer too small");
            }
            buf[..len].copy_from_slice(&ENV_SHELL[..len]);
            Ok(len)
        }
        _ => Err("Environment variable not found"),
    }
}

/// Sets or updates the string value of an environment variable.
pub unsafe fn set_env_var(name: &str, value: &str) -> Result<(), &'static str> {
    let val_bytes = value.as_bytes();
    match name {
        "PATH" => {
            if val_bytes.len() > 64 {
                return Err("Value too long");
            }
            ENV_PATH[..val_bytes.len()].copy_from_slice(val_bytes);
            ENV_PATH_LEN = val_bytes.len();
            Ok(())
        }
        "SHELL" => {
            if val_bytes.len() > 32 {
                return Err("Value too long");
            }
            ENV_SHELL[..val_bytes.len()].copy_from_slice(val_bytes);
            ENV_SHELL_LEN = val_bytes.len();
            Ok(())
        }
        _ => Err("Invalid environment variable key"),
    }
}
