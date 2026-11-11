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

//! Task control block (TCB) types, CPU execution contexts and lifecycle states.

pub mod context;
pub mod fd;
pub mod task;

#[cfg(test)]
mod tests;

pub use context::InterruptContext;
pub use fd::{FileDescriptor, MAX_FDS};
pub use task::{Task, TaskState, MAX_TASKS};
