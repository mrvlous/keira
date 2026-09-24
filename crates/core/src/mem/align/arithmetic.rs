// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Memory alignment arithmetic utilities for pages, frames, and structures.
//!
//! Provides bitwise alignment calculations ensuring pointer boundaries satisfy
//! hardware architecture requirements for MMU tables, DMA buffers, and SIMD registers.

/// Aligns a memory address upwards to the nearest specified alignment boundary.
///
/// # Invariants
///
/// The `align` parameter must be a non-zero power of two (e.g. 4, 8, 16, 4096).
/// Behavior is undefined if `align` is zero or not a power of two.
///
/// # Examples
///
/// ```
/// use keira_core::mem::align::align_up;
/// assert_eq!(align_up(0x1001, 4096), 0x2000);
/// assert_eq!(align_up(0x2000, 4096), 0x2000);
/// ```
#[inline(always)]
pub const fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}

/// Aligns a memory address downwards to the nearest specified alignment boundary.
///
/// # Invariants
///
/// The `align` parameter must be a non-zero power of two.
///
/// # Examples
///
/// ```
/// use keira_core::mem::align::align_down;
/// assert_eq!(align_down(0x1FFF, 4096), 0x1000);
/// assert_eq!(align_down(0x1000, 4096), 0x1000);
/// ```
#[inline(always)]
pub const fn align_down(addr: usize, align: usize) -> usize {
    addr & !(align - 1)
}

/// Checks whether a memory address satisfies the specified alignment boundary.
///
/// # Invariants
///
/// The `align` parameter must be a non-zero power of two.
#[inline(always)]
pub const fn is_aligned(addr: usize, align: usize) -> bool {
    (addr & (align - 1)) == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_align_up() {
        assert_eq!(align_up(0, 4096), 0);
        assert_eq!(align_up(1, 4096), 4096);
        assert_eq!(align_up(4095, 4096), 4096);
        assert_eq!(align_up(4096, 4096), 4096);
        assert_eq!(align_up(4097, 4096), 8192);
    }

    #[test]
    fn test_align_down() {
        assert_eq!(align_down(0, 4096), 0);
        assert_eq!(align_down(1, 4096), 0);
        assert_eq!(align_down(4095, 4096), 0);
        assert_eq!(align_down(4096, 4096), 4096);
        assert_eq!(align_down(4097, 4096), 4096);
    }

    #[test]
    fn test_is_aligned() {
        assert!(is_aligned(0, 4096));
        assert!(!is_aligned(1, 4096));
        assert!(is_aligned(4096, 4096));
        assert!(is_aligned(8192, 4096));
        assert!(!is_aligned(8193, 4096));
    }
}
