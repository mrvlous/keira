#![allow(unused_variables, unused_unsafe)]
// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Keira Kernel: Shell Command 'initrd'
//!
//! Implementation of the 'initrd' shell command.

pub fn run(parts: &mut core::str::SplitWhitespace) {
    unsafe {
        crate::fs::tar::list_files();
    }
}
