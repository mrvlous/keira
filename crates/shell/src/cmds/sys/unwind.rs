// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

#![allow(unused_variables, unused_unsafe)]

//!
//! Trigger kernel stack frame unwinder backtrace (Syscall 37).

#[inline(never)]
pub fn run(_parts: &mut core::str::SplitWhitespace) {
    keira_arch::unwind::unwind_stack();
}
