// SPDX-License-Identifier: GPL-2.0-only

//! Stack unwinding and callstack frame analysis subsystem.

pub mod unwind;

pub use unwind::{
    capture_backtrace, capture_from_frame, unwind_from_frame, unwind_stack, StackFrame,
};
