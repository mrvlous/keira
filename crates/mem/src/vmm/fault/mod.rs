// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Page Fault (#PF) interrupt dispatching, stack growth, and COW fault resolution.

pub mod handler;

pub use handler::{handle_page_fault, vmm_get_fault_stats, USER_STACK_BOTTOM, USER_STACK_TOP};
