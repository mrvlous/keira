// SPDX-License-Identifier: GPL-2.0-only

//! Seccomp system call dispatcher interface.

use keira_io::vga;

use super::filter::set_mode;
use crate::security::seccomp::model::{
    SeccompMode, SECCOMP_SET_MODE_FILTER, SECCOMP_SET_MODE_STRICT,
};

/// Enforce seccomp system call sandbox filter (Syscall 52).
pub fn sys_seccomp(op: u32, _flags: u32, _args_ptr: u64) -> Result<u64, &'static str> {
    match op {
        SECCOMP_SET_MODE_STRICT => {
            set_mode(SeccompMode::Strict);
            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("[SECCOMP] Strict Mode Sandbox Enforced\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            Ok(0)
        }
        SECCOMP_SET_MODE_FILTER => {
            set_mode(SeccompMode::Filter);
            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("[SECCOMP] Filter Mode Sandbox Enforced\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            Ok(0)
        }
        _ => Err("Invalid seccomp operation"),
    }
}
