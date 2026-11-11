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

//! System call numbers and vector classifications supported by Keira Kernel.

pub mod numbers;
pub mod vectors;

#[cfg(test)]
mod tests;

pub use numbers::*;
pub use vectors::{
    is_io_syscall, is_ipc_syscall, is_memory_syscall, is_process_syscall, is_valid_syscall,
    syscall_name,
};
