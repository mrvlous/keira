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

pub mod dispatch;
pub mod dump;
pub mod panic;
pub mod signals;

pub use dispatch::{exception_dispatcher, get_cpu_exception_count, TOTAL_CPU_EXCEPTIONS};
pub use dump::{write_core_dump, DumpWriter};
pub use panic::{panic_exception_dump, print_decimal_serial, print_hex, print_hex_serial};
pub use signals::{exception_name, exception_vector_to_signal, signal_name};
