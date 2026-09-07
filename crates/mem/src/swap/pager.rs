// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Anonymous physical memory swap space pager and partition slot manager.

/// Total number of 4KB swap slots managed (16,384 slots * 4KB = 64 MB).
pub const MAX_SWAP_SLOTS: usize = 16384;
pub const SWAP_BITMAP_WORDS: usize = MAX_SWAP_SLOTS / 64; // 256 words

static mut SWAP_ACTIVE: bool = false;
static mut SWAP_DEVICE: [u8; 64] = [0u8; 64];
static mut SWAP_DEVICE_LEN: usize = 0;
static mut SWAP_BITMAP: [u64; SWAP_BITMAP_WORDS] = [0u64; SWAP_BITMAP_WORDS];
static mut SWAP_TOTAL_PAGES: u64 = 0;
static mut SWAP_USED_PAGES: u64 = 0;
static mut SWAP_IN_COUNT: u64 = 0;
static mut SWAP_OUT_COUNT: u64 = 0;

/// Snapshot of virtual memory swap manager metrics.
#[derive(Copy, Clone)]
pub struct SwapStats {
    pub active: bool,
    pub device: [u8; 64],
    pub device_len: usize,
    pub total_pages: u64,
    pub used_pages: u64,
    pub free_pages: u64,
    pub swap_in_count: u64,
    pub swap_out_count: u64,
}

/// Activate swap space on target disk device partition or swap file.
pub fn swapon(path: &str, _swapflags: i32) -> Result<(), &'static str> {
    unsafe {
        if SWAP_ACTIVE {
            return Err("Swap space is already active");
        }

        let bytes = path.as_bytes();
        let copy_len = core::cmp::min(bytes.len(), 63);
        SWAP_DEVICE = [0u8; 64];
        SWAP_DEVICE[..copy_len].copy_from_slice(&bytes[..copy_len]);
        SWAP_DEVICE_LEN = copy_len;

        SWAP_BITMAP = [0u64; SWAP_BITMAP_WORDS];
        SWAP_TOTAL_PAGES = MAX_SWAP_SLOTS as u64;
        SWAP_USED_PAGES = 0;
        SWAP_IN_COUNT = 0;
        SWAP_OUT_COUNT = 0;
        SWAP_ACTIVE = true;
    }
    Ok(())
}

/// Syscall alias for swapon (Syscall 53).
///
/// # Safety
/// The caller must ensure `path_ptr` points to a valid null-terminated C string or is null.
pub unsafe fn sys_swapon(path_ptr: *const u8, swapflags: i32) -> Result<u64, &'static str> {
    let path = if !path_ptr.is_null() {
        let mut len = 0;
        while len < 64 && *path_ptr.add(len) != 0 {
            len += 1;
        }
        core::str::from_utf8(core::slice::from_raw_parts(path_ptr, len)).unwrap_or("/data/swapfile")
    } else {
        "/data/swapfile"
    };

    swapon(path, swapflags).map(|_| 0)
}

/// Deactivate swap space partition.
pub fn swapoff(_path: Option<&str>) -> Result<(), &'static str> {
    unsafe {
        if !SWAP_ACTIVE {
            return Err("Swap space is not active");
        }

        SWAP_ACTIVE = false;
        SWAP_BITMAP = [0u64; SWAP_BITMAP_WORDS];
        SWAP_USED_PAGES = 0;
        SWAP_TOTAL_PAGES = 0;
    }
    Ok(())
}

/// Syscall alias for swapoff (Syscall 54).
///
/// # Safety
/// The caller must ensure `path_ptr` points to a valid null-terminated C string or is null.
pub unsafe fn sys_swapoff(path_ptr: *const u8) -> Result<u64, &'static str> {
    let path = if !path_ptr.is_null() {
        let mut len = 0;
        while len < 64 && *path_ptr.add(len) != 0 {
            len += 1;
        }
        core::str::from_utf8(core::slice::from_raw_parts(path_ptr, len)).ok()
    } else {
        None
    };

    swapoff(path).map(|_| 0)
}

/// Check if swap partition is active.
pub fn is_active() -> bool {
    unsafe { SWAP_ACTIVE }
}

/// Allocate a free 4KB swap slot from the active swap space.
pub fn alloc_swap_slot() -> Option<usize> {
    unsafe {
        if !SWAP_ACTIVE || SWAP_USED_PAGES >= SWAP_TOTAL_PAGES {
            return None;
        }

        for word_idx in 0..SWAP_BITMAP_WORDS {
            let word = SWAP_BITMAP[word_idx];
            if word != u64::MAX {
                let bit_idx = (!word).trailing_zeros() as usize;
                SWAP_BITMAP[word_idx] |= 1u64 << bit_idx;
                SWAP_USED_PAGES += 1;
                SWAP_OUT_COUNT += 1;
                return Some(word_idx * 64 + bit_idx);
            }
        }
    }
    None
}

/// Release a previously allocated swap slot back to the free pool.
pub fn free_swap_slot(slot: usize) -> Result<(), &'static str> {
    unsafe {
        if !SWAP_ACTIVE {
            return Err("Swap is not active");
        }
        if slot >= MAX_SWAP_SLOTS {
            return Err("Invalid swap slot index");
        }

        let word_idx = slot / 64;
        let bit_idx = slot % 64;
        let mask = 1u64 << bit_idx;

        if (SWAP_BITMAP[word_idx] & mask) == 0 {
            return Err("Swap slot was not allocated");
        }

        SWAP_BITMAP[word_idx] &= !mask;
        if SWAP_USED_PAGES > 0 {
            SWAP_USED_PAGES -= 1;
        }
        SWAP_IN_COUNT += 1;
        Ok(())
    }
}

/// Query real-time swap manager statistics.
pub fn swap_stats() -> SwapStats {
    unsafe {
        let free = SWAP_TOTAL_PAGES.saturating_sub(SWAP_USED_PAGES);
        SwapStats {
            active: SWAP_ACTIVE,
            device: SWAP_DEVICE,
            device_len: SWAP_DEVICE_LEN,
            total_pages: SWAP_TOTAL_PAGES,
            used_pages: SWAP_USED_PAGES,
            free_pages: free,
            swap_in_count: SWAP_IN_COUNT,
            swap_out_count: SWAP_OUT_COUNT,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_swap_lifecycle_and_allocation() {
        // Ensure starting clean
        let _ = swapoff(None);
        assert!(!is_active());
        assert_eq!(alloc_swap_slot(), None);

        // Turn on swap
        assert!(swapon("/dev/sda2", 0).is_ok());
        assert!(is_active());

        let stats = swap_stats();
        assert_eq!(stats.total_pages, MAX_SWAP_SLOTS as u64);
        assert_eq!(stats.used_pages, 0);
        assert_eq!(stats.free_pages, MAX_SWAP_SLOTS as u64);

        // Allocate slots
        let s0 = alloc_swap_slot().expect("slot 0");
        let s1 = alloc_swap_slot().expect("slot 1");
        let s2 = alloc_swap_slot().expect("slot 2");

        assert_eq!(s0, 0);
        assert_eq!(s1, 1);
        assert_eq!(s2, 2);

        let stats_after = swap_stats();
        assert_eq!(stats_after.used_pages, 3);
        assert_eq!(stats_after.free_pages, (MAX_SWAP_SLOTS - 3) as u64);

        // Free slot 1
        assert!(free_swap_slot(s1).is_ok());
        // Double free should fail
        assert!(free_swap_slot(s1).is_err());

        // Next allocation should re-use slot 1
        let s1_reuse = alloc_swap_slot().expect("reuse slot 1");
        assert_eq!(s1_reuse, 1);

        // Free all
        assert!(free_swap_slot(s0).is_ok());
        assert!(free_swap_slot(s1_reuse).is_ok());
        assert!(free_swap_slot(s2).is_ok());

        // Deactivate swap
        assert!(swapoff(None).is_ok());
        assert!(!is_active());
        assert!(free_swap_slot(0).is_err());
    }
}
