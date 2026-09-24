// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Canonical architecture-independent memory sizes and layout constants.
//!
//! Defines physical and virtual page size boundaries, power-of-two memory units,
//! and page frame address calculation masks for bare-metal memory subsystems.

/// One byte in memory.
pub const BYTE: usize = 1;

/// One kibibyte (1024 bytes).
pub const KIB: usize = 1024 * BYTE;

/// One mebibyte (1024 kibibytes).
pub const MIB: usize = 1024 * KIB;

/// One gibibyte (1024 mebibytes).
pub const GIB: usize = 1024 * MIB;

/// Standard small page size (4 KiB) across x86 and x86_64 architectures.
pub const PAGE_SIZE_4K: usize = 4 * KIB;

/// Large page size (2 MiB) used with PAE or Long Mode page tables.
pub const PAGE_SIZE_2M: usize = 2 * MIB;

/// Huge page size (1 GiB) supported by modern x86_64 processor architectures.
pub const PAGE_SIZE_1G: usize = GIB;

/// Bit shift required to convert between byte addresses and 4 KiB page frame numbers.
pub const PAGE_SHIFT_4K: usize = 12;

/// Bit shift required to convert between byte addresses and 2 MiB large page numbers.
pub const PAGE_SHIFT_2M: usize = 21;

/// Bit shift required to convert between byte addresses and 1 GiB huge page numbers.
pub const PAGE_SHIFT_1G: usize = 30;

/// Mask extracting the 12-bit intra-page offset within a standard 4 KiB page frame.
pub const PAGE_OFFSET_MASK_4K: usize = PAGE_SIZE_4K - 1;

/// Mask extracting the 21-bit intra-page offset within a 2 MiB large page frame.
pub const PAGE_OFFSET_MASK_2M: usize = PAGE_SIZE_2M - 1;
