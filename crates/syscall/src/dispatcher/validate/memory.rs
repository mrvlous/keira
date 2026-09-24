// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Memory bounds constants and validation limits for process heap.

#[cfg(target_arch = "x86_64")]
pub const HEAP_MAX_VADDR: u64 = 0x7000_0000_0000;

#[cfg(target_arch = "x86")]
pub const HEAP_MAX_VADDR: u64 = 0x0400_0000;
