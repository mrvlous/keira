// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Physical memory page frame types, limits, and fundamental constants.

/// Standard 4 KiB x86_64 page and physical frame size.
pub const PAGE_SIZE: u64 = 4096;

/// 4 KiB page size alias.
pub const PAGE_SIZE_4K: u64 = 4096;

/// 1 MiB physical memory base address marking the boundary of low memory.
pub const KERNEL_BASE_1MB: u64 = 0x100000;

/// Maximum physical address supported by the 128 KiB frame bitmap (4 GiB = `0x1_0000_0000`).
pub const MAX_PHYS_ADDR_LIMIT: u64 = 4 * 1024 * 1024 * 1024;

/// Total number of 4 KiB page frames tracked by the physical memory allocator (1,048,576 frames).
pub const MAX_TRACKED_FRAMES: usize = (MAX_PHYS_ADDR_LIMIT / PAGE_SIZE) as usize;

/// Total number of 64-bit words required to represent the bitmap for all tracked frames (16,384 words = 128 KiB).
pub const BITMAP_WORDS: usize = MAX_TRACKED_FRAMES / 64;

/// Maximum number of distinct usable memory segments parsed from system bootloader descriptors.
pub const MAX_REGIONS: usize = 16;
