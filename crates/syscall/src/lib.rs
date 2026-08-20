// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

#![no_std]

//! System call routing, exception dispatching, and TSS configuration.

pub mod dispatcher;
pub mod exception;
pub mod table;
pub mod tss;

pub use dispatcher::{read_user_string, syscall_dispatcher, validate_fd, validate_user_ptr};
pub use exception::{exception_dispatcher, ExceptionStackFrame};
pub use table::*;
pub use tss::{init_user_mode, set_kernel_stack, TaskStateSegment, TSS};
