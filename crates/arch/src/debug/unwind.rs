// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Stack frame pointer walking and callstack backtrace unwinder.

#[cfg(target_os = "none")]
use core::arch::asm;

/// Captured instruction and frame pointer stack frame.
#[derive(Debug, Clone, Copy, Default)]
pub struct StackFrame {
    pub rip: u64,
    pub rbp: u64,
}

/// Walk active kernel stack frame pointers using inline frame pointer register.
#[inline(never)]
pub fn capture_backtrace(frames: &mut [StackFrame]) -> usize {
    #[cfg(not(target_os = "none"))]
    {
        let _ = frames;
        0
    }
    #[cfg(all(target_os = "none", target_arch = "x86_64"))]
    {
        let rbp: u64;
        unsafe { asm!("mov {}, rbp", out(reg) rbp) };
        let rip: u64;
        unsafe { asm!("lea {}, [rip]", out(reg) rip) };
        capture_from_frame(rbp, rip, frames)
    }
    #[cfg(all(target_os = "none", target_arch = "x86"))]
    {
        let ebp: u32;
        unsafe { asm!("mov {}, ebp", out(reg) ebp) };
        let eip: u32;
        unsafe { asm!("call 2f; 2: pop {}", out(reg) eip) };
        capture_from_frame(ebp as u64, eip as u64, frames)
    }
}

/// Walk kernel stack frame pointers from specific RBP/RIP context.
#[inline(never)]
pub fn capture_from_frame(
    starting_rbp: u64,
    starting_rip: u64,
    frames: &mut [StackFrame],
) -> usize {
    if frames.is_empty() {
        return 0;
    }

    frames[0] = StackFrame {
        rip: starting_rip,
        rbp: starting_rbp,
    };

    let mut rbp = starting_rbp;
    let mut depth = 1;

    while rbp != 0 && depth < frames.len() {
        let ptr_size = core::mem::size_of::<usize>() as u64;
        let next_rbp_ptr = rbp as *const usize;
        let rip_ptr = (rbp + ptr_size) as *const usize;

        if validate_ptr(rip_ptr as u64) && validate_ptr(next_rbp_ptr as u64) {
            let rip = unsafe { *rip_ptr } as u64;
            let next_rbp = unsafe { *next_rbp_ptr } as u64;

            frames[depth] = StackFrame { rip, rbp: next_rbp };
            depth += 1;

            if next_rbp <= rbp || next_rbp > 0x7FFFFFFFFFFF {
                break;
            }
            rbp = next_rbp;
        } else {
            break;
        }
    }
    depth
}

/// Unwind and capture callstack frames from given RBP/RIP frame pointer.
pub fn unwind_from_frame(rbp: u64, rip: u64) {
    let mut frames = [StackFrame { rip: 0, rbp: 0 }; 16];
    let _depth = capture_from_frame(rbp, rip, &mut frames);
}

/// Unwind current active kernel callstack.
pub fn unwind_stack() {
    let mut frames = [StackFrame { rip: 0, rbp: 0 }; 16];
    let _depth = capture_backtrace(&mut frames);
}

fn validate_ptr(ptr: u64) -> bool {
    let ptr_size = core::mem::size_of::<usize>() as u64;
    (0x100000..0x7FFFFFFFFFFF).contains(&ptr) && (ptr & (ptr_size - 1) == 0)
}
