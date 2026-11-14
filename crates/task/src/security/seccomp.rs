// SPDX-License-Identifier: GPL-2.0-only
/*
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra <mrvlous@proton.me>
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 */

//! Secure Computing (seccomp) syscall filtering and sandbox enforcement.

pub mod model {
    pub mod modes {
        pub const SECCOMP_SET_MODE_STRICT: u32 = 0;
        pub const SECCOMP_SET_MODE_FILTER: u32 = 1;
        
        #[derive(Debug, Copy, Clone, PartialEq, Eq)]
        pub enum SeccompMode {
            Disabled,
            Strict,
            Filter,
        }
        
        pub struct SeccompState {
            pub mode: SeccompMode,
            pub allowed_mask: [u64; 2],
            pub total_checked: u64,
            pub total_violations: u64,
            pub last_violation_syscall: u64,
        }
    }
    pub use self::modes::*;
}

pub mod sandbox {
    pub mod filter {
        use crate::security::seccomp::model::{SeccompMode, SeccompState};
        
        pub static mut SECCOMP_STATE: SeccompState = SeccompState {
            mode: SeccompMode::Disabled,
            allowed_mask: [0xFFFF_FFFF_FFFF_FFFF, 0xFFFF_FFFF_FFFF_FFFF],
            total_checked: 0,
            total_violations: 0,
            last_violation_syscall: 0,
        };
        
        pub static mut SECCOMP_STRICT_ACTIVE: bool = false;
        
        pub fn check_syscall(syscall_num: u64) -> bool {
            unsafe {
                let state = &mut SECCOMP_STATE;
                state.total_checked += 1;
        
                match state.mode {
                    SeccompMode::Disabled => true,
                    SeccompMode::Strict => {
                        // Strict mode: Only allow read (7, 15), write (1, 8, 16), exit (2), sigreturn (65) and seccomp (52)
                        let allowed = matches!(syscall_num, 1 | 2 | 7 | 8 | 15 | 16 | 52 | 65);
                        if !allowed {
                            state.total_violations += 1;
                            state.last_violation_syscall = syscall_num;
                        }
                        allowed
                    }
                    SeccompMode::Filter => {
                        let word_idx = (syscall_num / 64) as usize;
                        let bit_idx = (syscall_num % 64) as usize;
        
                        let allowed = if word_idx < state.allowed_mask.len() {
                            (state.allowed_mask[word_idx] & (1u64 << bit_idx)) != 0
                        } else {
                            false
                        };
        
                        if !allowed {
                            state.total_violations += 1;
                            state.last_violation_syscall = syscall_num;
                        }
                        allowed
                    }
                }
            }
        }
        
        pub fn set_mode(mode: SeccompMode) {
            unsafe {
                SECCOMP_STATE.mode = mode;
                SECCOMP_STRICT_ACTIVE = matches!(mode, SeccompMode::Strict);
            }
        }
        
        pub fn get_mode() -> SeccompMode {
            unsafe { SECCOMP_STATE.mode }
        }
        
        pub fn allow_syscall(syscall_num: u64) {
            unsafe {
                let word_idx = (syscall_num / 64) as usize;
                let bit_idx = (syscall_num % 64) as usize;
                if word_idx < SECCOMP_STATE.allowed_mask.len() {
                    SECCOMP_STATE.allowed_mask[word_idx] |= 1u64 << bit_idx;
                }
            }
        }
        
        pub fn deny_syscall(syscall_num: u64) {
            unsafe {
                let word_idx = (syscall_num / 64) as usize;
                let bit_idx = (syscall_num % 64) as usize;
                if word_idx < SECCOMP_STATE.allowed_mask.len() {
                    SECCOMP_STATE.allowed_mask[word_idx] &= !(1u64 << bit_idx);
                }
            }
        }
        
        pub fn is_syscall_allowed(syscall_num: u64) -> bool {
            unsafe {
                let word_idx = (syscall_num / 64) as usize;
                let bit_idx = (syscall_num % 64) as usize;
                if word_idx < SECCOMP_STATE.allowed_mask.len() {
                    (SECCOMP_STATE.allowed_mask[word_idx] & (1u64 << bit_idx)) != 0
                } else {
                    false
                }
            }
        }
        
        pub fn get_stats() -> (u64, u64, u64) {
            unsafe {
                (
                    SECCOMP_STATE.total_checked,
                    SECCOMP_STATE.total_violations,
                    SECCOMP_STATE.last_violation_syscall,
                )
            }
        }
        
        pub fn reset() {
            unsafe {
                SECCOMP_STATE.mode = SeccompMode::Disabled;
                SECCOMP_STATE.allowed_mask = [0xFFFF_FFFF_FFFF_FFFF, 0xFFFF_FFFF_FFFF_FFFF];
                SECCOMP_STATE.total_checked = 0;
                SECCOMP_STATE.total_violations = 0;
                SECCOMP_STATE.last_violation_syscall = 0;
                SECCOMP_STRICT_ACTIVE = false;
            }
        }
    }
    pub mod syscall {
        use keira_io::vga;
        
        use super::filter::set_mode;
        use crate::security::seccomp::model::{
            SeccompMode, SECCOMP_SET_MODE_FILTER, SECCOMP_SET_MODE_STRICT,
        };
        
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
    }
    pub use self::filter::*;
    pub use self::syscall::*;
}

pub use self::model::*;
pub use self::sandbox::*;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_seccomp_strict_and_filter_lifecycle() {
        reset();
        set_mode(SeccompMode::Strict);
        assert_eq!(get_mode(), SeccompMode::Strict);
    
        // Allowed: 1, 2, 7, 8, 15, 16, 52, 65
        assert!(check_syscall(1));
        assert!(check_syscall(2));
        assert!(check_syscall(7));
        assert!(check_syscall(15));
        assert!(check_syscall(52));
    
        // Denied in strict
        assert!(!check_syscall(10));
        assert!(!check_syscall(21)); // fork
    
        let (checked, violations, last_viol) = get_stats();
        assert!(checked > 0);
        assert_eq!(violations, 2);
        assert_eq!(last_viol, 21);
    
        reset();
        set_mode(SeccompMode::Filter);
    
        // Initially all allowed in bitmask
        assert!(check_syscall(20));
    
        // Deny syscall 20
        deny_syscall(20);
        assert!(!check_syscall(20));
    
        // Re-allow syscall 20
        allow_syscall(20);
        assert!(check_syscall(20));
    
        reset();
    }
}
