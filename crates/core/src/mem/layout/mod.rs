// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Canonical architecture memory layout and page size constants subsystem.

pub mod constants;

pub use constants::{
    BYTE, GIB, KIB, MIB, PAGE_OFFSET_MASK_2M, PAGE_OFFSET_MASK_4K, PAGE_SHIFT_1G, PAGE_SHIFT_2M,
    PAGE_SHIFT_4K, PAGE_SIZE_1G, PAGE_SIZE_2M, PAGE_SIZE_4K,
};
