// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Kernel diagnostic debugging, hardware breakpoints, and stack unwinding.
//!
//! Subdivided into dedicated modules for hardware debug registers (DR0-DR7)
//! and callstack frame pointer backtrace traversal.

pub mod breakpoint;
pub mod stack;

pub use breakpoint::{
    check_and_clear_status, clear_watchpoint, read_dr0, read_dr1, read_dr2, read_dr3, read_dr6,
    read_dr7, set_watchpoint, write_dr0, write_dr1, write_dr2, write_dr3, write_dr6, write_dr7,
    WatchpointCondition, WatchpointEntry, WatchpointSize, WATCHPOINTS,
};
pub use stack::{
    capture_backtrace, capture_from_frame, unwind_from_frame, unwind_stack, StackFrame,
};

/// Compatibility re-export module for `debug::unwind`.
pub mod unwind {
    pub use super::stack::*;
}
