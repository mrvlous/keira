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
