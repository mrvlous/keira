// SPDX-License-Identifier: GPL-2.0-only

//! Architecture-specific CPU exception stack frame pushed by hardware and ISR stubs.

pub mod types;

pub use types::ExceptionStackFrame;
