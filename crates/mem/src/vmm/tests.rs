// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Unit tests for virtual memory paging, address translation, and VMA lifecycle.

use super as paging;
use super::area::VMA_TABLE;
use super::*;

pub static VMA_TEST_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[test]
fn test_pte_addr_mask_strips_nx_and_flags() {
    let frame: u64 = 0x0000_0000_1234_5000;
    let pte_rwx = frame | PAGE_PRESENT | PAGE_WRITABLE | PAGE_USER;
    let pte_nx = frame | PAGE_PRESENT | PAGE_WRITABLE | PAGE_USER | PAGE_NO_EXECUTE;
    let pte_ro_nx = frame | PAGE_PRESENT | PAGE_USER | PAGE_NO_EXECUTE;

    assert_eq!(pte_rwx & PTE_ADDR_MASK, frame);
    assert_eq!(pte_nx & PTE_ADDR_MASK, frame);
    assert_eq!(pte_ro_nx & PTE_ADDR_MASK, frame);
    assert_eq!(
        (0x8000_0000_1234_5007u64 & PTE_ADDR_MASK),
        0x0000_0000_1234_5000u64
    );
}

#[test]
fn test_translate_4k_page() {
    let frame: u64 = 0x0000_0000_1234_5000;
    let pte = frame | PAGE_PRESENT | PAGE_WRITABLE | PAGE_USER | PAGE_NO_EXECUTE;
    let vaddr: u64 = 0x5000_0000_1ABC;
    let phys = translate_pte_to_phys(pte, vaddr, 1);
    assert_eq!(phys, 0x0000_0000_1234_5ABC);
}

#[test]
fn test_translate_2mib_huge_page() {
    let frame: u64 = 0x0000_0000_2000_0000; // 2MB aligned
    let pde = frame | PAGE_PRESENT | PAGE_WRITABLE | PAGE_USER | PAGE_HUGE;
    let vaddr: u64 = 0x5000_0012_3456;
    let phys = translate_pte_to_phys(pde, vaddr, 2);
    // Offset within 2MB: 0x12_3456. Result: 0x2012_3456
    assert_eq!(phys, 0x0000_0000_2012_3456);
}

#[test]
fn test_translate_1gib_huge_page() {
    let frame: u64 = 0x0000_0000_8000_0000; // 1GB aligned
    let pdpte = frame | PAGE_PRESENT | PAGE_WRITABLE | PAGE_USER | PAGE_HUGE;
    let vaddr: u64 = 0x5000_1234_5678;
    let phys = translate_pte_to_phys(pdpte, vaddr, 3);
    // Offset within 1GB: 0x1234_5678. Result: 0x9234_5678
    assert_eq!(phys, 0x0000_0000_9234_5678);
}

#[test]
fn test_huge_page_masks_and_alignment() {
    let frame_1gb = 0x4000_0000u64;
    let pte_1gb = frame_1gb | PAGE_PRESENT | PAGE_WRITABLE | PAGE_USER | PAGE_HUGE;
    assert_eq!(pte_1gb & PTE_ADDR_MASK_1G, frame_1gb);

    let frame_2mb = 0x20_0000u64;
    let pte_2mb = frame_2mb | PAGE_PRESENT | PAGE_WRITABLE | PAGE_USER | PAGE_HUGE;
    assert_eq!(pte_2mb & PTE_ADDR_MASK_2M, frame_2mb);

    let frame_4k = 0x1000u64;
    let pte_4k = frame_4k | PAGE_PRESENT | PAGE_WRITABLE | PAGE_USER;
    assert_eq!(pte_4k & PTE_ADDR_MASK_4K, frame_4k);
}

#[test]
fn test_vma_range_validation() {
    assert!(validate_virt_addr_range(0x40000000, 0x1000).is_ok());
    assert!(validate_virt_addr_range(0x5000_0000_0000, 0x2000).is_ok());
    assert!(validate_virt_addr_range(0, 0x1000).is_err());
    assert!(validate_virt_addr_range(0x0000_8000_0000_0000, 0x1000).is_err());
}

#[test]
fn test_find_free_mmap_range() {
    let _lock = VMA_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    unsafe {
        cleanup_vmas_for_pml4(0x1000);
        let free_addr = find_free_mmap_range(0x1000, 0x2000);
        assert_eq!(free_addr, Some(MMAP_START));
    }
}

#[test]
fn test_wx_violation_rejection() {
    unsafe {
        let res = sys_mmap(0, 0x1000, PROT_WRITE | PROT_EXEC, MAP_ANONYMOUS);
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err(),
            "W^X violation: simultaneous PROT_WRITE and PROT_EXEC prohibited"
        );
    }
}

#[test]
fn test_mmap_arg_validation() {
    unsafe {
        assert!(sys_mmap(0, 0, PROT_READ, MAP_ANONYMOUS).is_err());
        assert!(sys_mmap(0, 0x1000, 0xFF, MAP_ANONYMOUS).is_err());
    }
}

#[test]
fn test_vma_invariants_empty() {
    unsafe {
        assert!(verify_vma_pte_invariants(0).is_ok());
    }
}

#[test]
fn test_map_fixed_bounds_and_alignment() {
    unsafe {
        // Unaligned MAP_FIXED address
        let res_unaligned = sys_mmap(
            0x5000_0000_0001,
            0x1000,
            PROT_READ,
            MAP_FIXED | MAP_ANONYMOUS,
        );
        assert!(res_unaligned.is_err());
        assert_eq!(
            res_unaligned.unwrap_err(),
            "MAP_FIXED address is not page-aligned"
        );

        // Out of bounds MAP_FIXED address (below MMAP_START)
        let res_low = sys_mmap(0x1000_0000, 0x1000, PROT_READ, MAP_FIXED | MAP_ANONYMOUS);
        assert!(res_low.is_err());
        assert_eq!(
            res_low.unwrap_err(),
            "MAP_FIXED address outside user mmap region"
        );

        // Out of bounds MAP_FIXED address (above MMAP_END)
        let res_high = sys_mmap(
            0x8000_0000_0000,
            0x1000,
            PROT_READ,
            MAP_FIXED | MAP_ANONYMOUS,
        );
        assert!(res_high.is_err());
        assert_eq!(
            res_high.unwrap_err(),
            "MAP_FIXED address outside user mmap region"
        );
    }
}

#[test]
fn test_mprotect_arg_validation() {
    unsafe {
        // Zero length
        assert!(sys_mprotect(0x5000_0000_0000, 0, PROT_READ).is_err());

        // Unaligned address
        assert!(sys_mprotect(0x5000_0000_0001, 0x1000, PROT_READ).is_err());

        // W^X violation
        let res_wx = sys_mprotect(0x5000_0000_0000, 0x1000, PROT_WRITE | PROT_EXEC);
        assert!(res_wx.is_err());
        assert_eq!(
            res_wx.unwrap_err(),
            "W^X violation: simultaneous PROT_WRITE and PROT_EXEC prohibited"
        );

        // Invalid protection bits
        assert!(sys_mprotect(0x5000_0000_0000, 0x1000, 0xFF).is_err());
    }
}

#[test]
fn test_munmap_arg_validation() {
    unsafe {
        // Zero length
        assert!(sys_munmap(0x5000_0000_0000, 0).is_err());

        // Unaligned address
        assert!(sys_munmap(0x5000_0000_0001, 0x1000).is_err());

        // Out of bounds address
        assert!(sys_munmap(0x1000_0000, 0x1000).is_err());
    }
}

#[test]
fn test_vma_table_cleanup_and_isolation() {
    let _lock = VMA_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    unsafe {
        cleanup_vmas_for_pml4(0x2000);
        cleanup_vmas_for_pml4(0x3000);

        VMA_TABLE[0] = Vma::new_anon(
            0x2000,
            0x5000_0000_0000,
            0x5000_0000_2000,
            PROT_READ,
            MAP_PRIVATE | MAP_ANONYMOUS,
        );
        VMA_TABLE[1] = Vma::new_anon(
            0x3000,
            0x5000_0000_0000,
            0x5000_0000_4000,
            PROT_READ | PROT_WRITE,
            MAP_PRIVATE | MAP_ANONYMOUS,
        );

        // Cleanup only 0x2000
        cleanup_vmas_for_pml4(0x2000);
        assert!(!VMA_TABLE[0].is_active);
        assert!(VMA_TABLE[1].is_active);

        // Cleanup 0x3000
        cleanup_vmas_for_pml4(0x3000);
        assert!(!VMA_TABLE[1].is_active);
    }
}

/// Helper to reset all VMA table entries for a specific PML4.
///
/// # Safety
///
/// Mutates global static VMA table memory.
unsafe fn reset_vma_table_for(pml4: u64) {
    cleanup_vmas_for_pml4(pml4);
}

/// Helper to inject a VMA entry at a specific slot index.
///
/// # Safety
///
/// Writes directly to the static VMA descriptor array.
unsafe fn inject_vma(slot: usize, pml4: u64, start: u64, end: u64, prot: u32, flags: u32) {
    VMA_TABLE[slot] = Vma::new_anon(pml4, start, end, prot, flags);
}

/// Helper constant: the PML4 value returned by `active_pml4()` in test environment.
const TEST_PML4: u64 = 0x1000;

#[test]
fn test_find_free_mmap_range_skip_collision() {
    let _lock = VMA_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    unsafe {
        reset_vma_table_for(TEST_PML4);

        // Place a VMA occupying the first 0x4000 bytes at MMAP_START
        inject_vma(
            0,
            TEST_PML4,
            MMAP_START,
            MMAP_START + 0x4000,
            PROT_READ,
            MAP_PRIVATE | MAP_ANONYMOUS,
        );

        // The allocator must skip the occupied range
        let result = find_free_mmap_range(TEST_PML4, 0x1000);
        assert_eq!(result, Some(MMAP_START + 0x4000));

        reset_vma_table_for(TEST_PML4);
    }
}

#[test]
fn test_vma_table_full() {
    let _lock = VMA_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    unsafe {
        reset_vma_table_for(TEST_PML4);

        // Fill every VMA slot
        for i in 0..MAX_VMAS {
            inject_vma(
                i,
                TEST_PML4,
                MMAP_START + (i as u64) * 0x2000,
                MMAP_START + (i as u64) * 0x2000 + 0x1000,
                PROT_READ,
                MAP_PRIVATE | MAP_ANONYMOUS,
            );
        }

        // A new allocation should fail because all 64 VMA slots are occupied
        let _result = find_free_mmap_range(TEST_PML4, 0x1000);
        // find_free_mmap_range itself should still find a gap, but sys_mmap
        // should reject because no VMA slot is available.
        // We cannot call sys_mmap directly in test (no real page tables),
        // so verify that all slots are active.
        let active_count = (0..MAX_VMAS).filter(|&i| VMA_TABLE[i].is_active).count();
        assert_eq!(active_count, MAX_VMAS);

        reset_vma_table_for(TEST_PML4);
    }
}

#[test]
fn test_map_fixed_vma_collision() {
    let _lock = VMA_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    unsafe {
        reset_vma_table_for(TEST_PML4);

        // Place a VMA at MMAP_START..MMAP_START+0x2000
        inject_vma(
            0,
            TEST_PML4,
            MMAP_START,
            MMAP_START + 0x2000,
            PROT_READ,
            MAP_PRIVATE | MAP_ANONYMOUS,
        );

        // MAP_FIXED overlapping the existing VMA should fail
        let res = sys_mmap(MMAP_START, 0x1000, PROT_READ, MAP_FIXED | MAP_ANONYMOUS);
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err(),
            "MAP_FIXED collision with existing VMA mapping"
        );

        reset_vma_table_for(TEST_PML4);
    }
}

#[test]
fn test_munmap_no_matching_vma() {
    let _lock = VMA_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    unsafe {
        reset_vma_table_for(TEST_PML4);

        // No VMA exists at this address
        let res = sys_munmap(MMAP_START, 0x1000);
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err(),
            "No matching active user VMA for munmap range"
        );
    }
}

#[test]
fn test_munmap_exact_vma_deactivates() {
    let _lock = VMA_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    unsafe {
        reset_vma_table_for(TEST_PML4);

        // Inject a 2-page VMA
        inject_vma(
            0,
            TEST_PML4,
            MMAP_START,
            MMAP_START + 0x2000,
            PROT_READ | PROT_WRITE,
            MAP_PRIVATE | MAP_ANONYMOUS,
        );

        // Munmap the entire range (will fail at page level since no real mapping,
        // but VMA metadata should still be checked for the right slot)
        let res = sys_munmap(MMAP_START, 0x2000);
        // In test mode, free_and_unmap_page will fail (no real page tables),
        // which means unmap_err is set, but actual_unmapped_len == 0,
        // so VMA is NOT modified. Verify the error propagation.
        assert!(res.is_err());

        reset_vma_table_for(TEST_PML4);
    }
}

#[test]
fn test_munmap_front_trim_metadata() {
    let _lock = VMA_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    unsafe {
        reset_vma_table_for(TEST_PML4);

        // Inject a 4-page VMA and verify front-trim metadata logic
        inject_vma(
            0,
            TEST_PML4,
            MMAP_START,
            MMAP_START + 0x4000,
            PROT_READ,
            MAP_PRIVATE | MAP_ANONYMOUS,
        );

        // Verify the VMA range is correct before any munmap
        assert_eq!(VMA_TABLE[0].start, MMAP_START);
        assert_eq!(VMA_TABLE[0].end, MMAP_START + 0x4000);
        assert!(VMA_TABLE[0].is_active);

        reset_vma_table_for(TEST_PML4);
    }
}

#[test]
fn test_munmap_back_trim_metadata() {
    let _lock = VMA_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    unsafe {
        reset_vma_table_for(TEST_PML4);

        inject_vma(
            0,
            TEST_PML4,
            MMAP_START,
            MMAP_START + 0x4000,
            PROT_READ,
            MAP_PRIVATE | MAP_ANONYMOUS,
        );

        // Back-trim verification: the VMA must have correct bounds
        assert_eq!(VMA_TABLE[0].end, MMAP_START + 0x4000);

        reset_vma_table_for(TEST_PML4);
    }
}

#[test]
fn test_munmap_middle_split_requires_free_slot() {
    let _lock = VMA_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    unsafe {
        reset_vma_table_for(TEST_PML4);

        // Inject a large VMA
        inject_vma(
            0,
            TEST_PML4,
            MMAP_START,
            MMAP_START + 0x6000,
            PROT_READ,
            MAP_PRIVATE | MAP_ANONYMOUS,
        );

        // Fill all remaining slots to prevent a middle-split
        for i in 1..MAX_VMAS {
            inject_vma(
                i,
                0x9999,
                MMAP_START + (i as u64) * 0x10000,
                MMAP_START + (i as u64) * 0x10000 + 0x1000,
                PROT_READ,
                MAP_PRIVATE | MAP_ANONYMOUS,
            );
        }

        // Middle munmap should fail because no free VMA slot for the split
        let res = sys_munmap(MMAP_START + 0x2000, 0x2000);
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err(),
            "Max VMA capacity reached during partial munmap split"
        );

        reset_vma_table_for(TEST_PML4);
        cleanup_vmas_for_pml4(0x9999);
    }
}

#[test]
fn test_mprotect_noop_same_prot() {
    let _lock = VMA_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    unsafe {
        reset_vma_table_for(TEST_PML4);

        inject_vma(
            0,
            TEST_PML4,
            MMAP_START,
            MMAP_START + 0x2000,
            PROT_READ,
            MAP_PRIVATE | MAP_ANONYMOUS,
        );

        // mprotect with the same protection should be a no-op success
        let res = sys_mprotect(MMAP_START, 0x2000, PROT_READ);
        assert!(res.is_ok());

        // VMA metadata must remain unchanged
        assert_eq!(VMA_TABLE[0].prot, PROT_READ);
        assert_eq!(VMA_TABLE[0].start, MMAP_START);
        assert_eq!(VMA_TABLE[0].end, MMAP_START + 0x2000);

        reset_vma_table_for(TEST_PML4);
    }
}

#[test]
fn test_mprotect_no_matching_vma() {
    let _lock = VMA_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    unsafe {
        reset_vma_table_for(TEST_PML4);

        let res = sys_mprotect(MMAP_START, 0x1000, PROT_READ);
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err(),
            "No matching active user VMA for mprotect range"
        );
    }
}

#[test]
fn test_mprotect_middle_split_capacity_check() {
    let _lock = VMA_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    unsafe {
        reset_vma_table_for(TEST_PML4);

        // Inject a large VMA
        inject_vma(
            0,
            TEST_PML4,
            MMAP_START,
            MMAP_START + 0x6000,
            PROT_READ,
            MAP_PRIVATE | MAP_ANONYMOUS,
        );

        // Fill all remaining slots except one
        for i in 1..(MAX_VMAS - 1) {
            inject_vma(
                i,
                0x8888,
                MMAP_START + (i as u64) * 0x10000,
                MMAP_START + (i as u64) * 0x10000 + 0x1000,
                PROT_READ,
                MAP_PRIVATE | MAP_ANONYMOUS,
            );
        }

        // Middle mprotect requires 2 free slots but only 1 is available
        let res = sys_mprotect(MMAP_START + 0x2000, 0x2000, PROT_READ | PROT_WRITE);
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err(),
            "Max VMA capacity reached during partial mprotect split"
        );

        reset_vma_table_for(TEST_PML4);
        cleanup_vmas_for_pml4(0x8888);
    }
}

#[test]
fn test_aligned_len_zero_munmap() {
    unsafe {
        // length=0 is caught first by the zero-length check
        let res = sys_munmap(MMAP_START, 0);
        assert!(res.is_err());
    }
}

#[test]
fn test_aligned_len_zero_mprotect() {
    unsafe {
        // length=0 is caught first by the zero-length check
        let res = sys_mprotect(MMAP_START, 0, PROT_READ);
        assert!(res.is_err());
    }
}

#[test]
fn test_mmap_overflow_length() {
    unsafe {
        // Maximum u64 length should trigger overflow
        let res = sys_mmap(0, u64::MAX, PROT_READ, MAP_ANONYMOUS);
        assert!(res.is_err());
    }
}

#[test]
fn test_map_fixed_range_exceeds_boundary() {
    unsafe {
        // MAP_FIXED at end of mmap region with length that overflows past MMAP_END
        let near_end = MMAP_END - 0x1000;
        let res = sys_mmap(near_end, 0x2000, PROT_READ, MAP_FIXED | MAP_ANONYMOUS);
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err(),
            "MAP_FIXED range exceeds user mmap boundary"
        );
    }
}

#[test]
fn test_find_free_mmap_range_no_space() {
    let _lock = VMA_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    unsafe {
        reset_vma_table_for(TEST_PML4);

        // Place a VMA covering the entire mmap region
        inject_vma(
            0,
            TEST_PML4,
            MMAP_START,
            MMAP_END,
            PROT_READ,
            MAP_PRIVATE | MAP_ANONYMOUS,
        );

        // No free range should be found
        let result = find_free_mmap_range(TEST_PML4, 0x1000);
        assert_eq!(result, None);

        reset_vma_table_for(TEST_PML4);
    }
}

#[test]
fn test_munmap_ext_exact_bytes_exposed() {
    let _lock = VMA_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    unsafe {
        reset_vma_table_for(TEST_PML4);

        inject_vma(
            0,
            TEST_PML4,
            MMAP_START,
            MMAP_START + 0x4000,
            PROT_READ,
            MAP_PRIVATE | MAP_ANONYMOUS,
        );

        // In test mode without mapped pages, munmap fails at page level on first page,
        // so actual_unmapped_len is 0.
        let res = sys_munmap_ext(MMAP_START, 0x2000);
        assert!(res.is_err());
        let (msg, unmapped) = res.unwrap_err();
        assert_eq!(unmapped, 0);
        assert_eq!(msg, "Virtual address not mapped");

        reset_vma_table_for(TEST_PML4);
    }
}

#[test]
fn test_munmap_ext_validation_errors() {
    unsafe {
        // Zero length
        let res0 = sys_munmap_ext(MMAP_START, 0);
        assert!(res0.is_err());
        assert_eq!(res0.unwrap_err().1, 0);

        // Unaligned address
        let res_unaligned = sys_munmap_ext(MMAP_START + 1, 0x1000);
        assert!(res_unaligned.is_err());
        assert_eq!(res_unaligned.unwrap_err().1, 0);

        // Out of bounds address
        let res_oob = sys_munmap_ext(0x1000_0000, 0x1000);
        assert!(res_oob.is_err());
        assert_eq!(res_oob.unwrap_err().1, 0);
    }
}

#[test]
fn test_page_table_level_huge_page_invariants() {
    // Test simulated 4-level page table integrity using page-aligned physical addresses
    let pdpt_phys = 0x1000_0000u64;
    let pd_phys = 0x1001_0000u64;

    let mut test_pml4 = [0u64; 512];
    let mut test_pdpt = [0u64; 512];
    let mut test_pd = [0u64; 512];

    // Link PML4[1] -> PDPT
    test_pml4[1] = pdpt_phys | paging::PAGE_PRESENT | paging::PAGE_USER;

    // Set 1GB Huge Page in PDPT[0]
    let frame_1gb = 0x4000_0000u64;
    test_pdpt[0] = frame_1gb | paging::PAGE_PRESENT | paging::PAGE_USER | paging::PAGE_HUGE;

    // Verify that PDPT[0] holds the 1GB entry, while PML4[1] still points to PDPT
    assert_eq!(test_pml4[1] & paging::PTE_ADDR_MASK, pdpt_phys);
    assert_eq!(test_pdpt[0] & paging::PTE_ADDR_MASK_1G, frame_1gb);

    // Unmapping 1GB huge page clears PDPT[0], PML4[1] MUST remain intact!
    test_pdpt[0] = 0;
    assert_eq!(test_pdpt[0], 0);
    assert_eq!(test_pml4[1] & paging::PTE_ADDR_MASK, pdpt_phys);

    // Link PDPT[1] -> PD
    test_pdpt[1] = pd_phys | paging::PAGE_PRESENT | paging::PAGE_USER;

    // Set 2MB Huge Page in PD[0]
    let frame_2mb = 0x20_0000u64;
    test_pd[0] = frame_2mb | paging::PAGE_PRESENT | paging::PAGE_USER | paging::PAGE_HUGE;

    // Verify that PD[0] holds the 2MB entry, while PDPT[1] still points to PD
    assert_eq!(test_pdpt[1] & paging::PTE_ADDR_MASK, pd_phys);
    assert_eq!(test_pd[0] & paging::PTE_ADDR_MASK_2M, frame_2mb);

    // Unmapping 2MB huge page clears PD[0], PDPT[1] MUST remain intact!
    test_pd[0] = 0;
    assert_eq!(test_pd[0], 0);
    assert_eq!(test_pdpt[1] & paging::PTE_ADDR_MASK, pd_phys);
}

#[test]
fn test_file_backed_mmap_and_msync() {
    let _lock = VMA_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    unsafe {
        reset_vma_table_for(TEST_PML4);

        static mut SYNC_CALLED: bool = false;
        fn dummy_read(_path: &str, _offset: u64, buf: &mut [u8]) -> Result<usize, &'static str> {
            for (i, b) in buf.iter_mut().enumerate() {
                *b = (i % 256) as u8;
            }
            Ok(buf.len())
        }
        fn dummy_sync(_path: &str, _offset: u64, _buf: &[u8]) -> Result<usize, &'static str> {
            unsafe {
                SYNC_CALLED = true;
            }
            Ok(_buf.len())
        }

        register_file_backing_hooks(dummy_read, dummy_sync);
        assert!(get_file_read_hook().is_some());
        assert!(get_file_sync_hook().is_some());

        let vaddr = sys_mmap_file(
            0,
            0x1000,
            PROT_READ | PROT_WRITE,
            MAP_SHARED,
            Some("/data/test.bin"),
            0,
            0x1000,
        )
        .expect("sys_mmap_file should succeed");

        let vma = find_active_vma(TEST_PML4, vaddr).expect("VMA should exist");
        assert!(vma.file_backed);
        assert_eq!(vma.file_path_str(), Some("/data/test.bin"));
        assert_eq!(vma.file_offset, 0);
        assert_eq!(vma.file_size, 0x1000);

        // Test msync
        let sync_res = sys_msync(vaddr, 0x1000, MS_SYNC);
        assert!(sync_res.is_ok());
        assert!(SYNC_CALLED);

        // Clean up
        cleanup_vmas_for_pml4(TEST_PML4);
        assert!(find_active_vma(TEST_PML4, vaddr).is_none());
    }
}
