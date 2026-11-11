// SPDX-License-Identifier: GPL-2.0-only

//! Rust exception dispatcher for CPU exceptions, signal delivery and core dump generation.

pub mod frame;
pub mod handler;

#[cfg(test)]
mod tests;

pub use frame::ExceptionStackFrame;
pub use handler::{
    exception_dispatcher, exception_name, exception_vector_to_signal, get_cpu_exception_count,
    signal_name, write_core_dump, TOTAL_CPU_EXCEPTIONS,
};
