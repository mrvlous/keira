// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Unit tests for physical memory frame allocation, boundary checks, and invariant assertions.

use super::frame::bitmap::*;
use super::region::descriptor::*;
use super::sync::guard::*;
use super::*;
use core::sync::atomic::Ordering;

#[test]
fn test_is_valid_ram_range_checks() {
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    reset_pmm_stats();
    unsafe {
        REGIONS[0] = UsableRegion {
            start: 0x100000,
            end: 0x8000000,
            current: 0x100000,
        };
        REGION_COUNT = 1;
        TOTAL_USABLE_RAM = 0x8000000;
        MAX_PHYS_ADDR = 0x8000000;

        assert!(is_valid_ram_range(0x100000, 0x200000));
        assert!(is_valid_ram_range(0x2000000, 0x4000000));

        assert!(!is_valid_ram_range(0x0, 0x1000));
        assert!(!is_valid_ram_range(0x80000, 0x1000));

        assert!(!is_valid_ram_range(0x100001, 0x1000));

        assert!(!is_valid_ram_range(0x100000, 0));

        assert!(!is_valid_ram_range(0x7F00000, 0x200000));
        assert!(!is_valid_ram_range(0x8000000, 0x1000));

        assert!(!is_valid_ram_range(0xFFFF_FFFF_FFFF_F000, 0x2000));
    }
}

#[test]
fn test_is_valid_ram_range_early_boot_fallback() {
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    reset_pmm_stats();
    unsafe {
        REGION_COUNT = 0;
        MAX_PHYS_ADDR = 0x8000_0000;

        assert!(is_valid_ram_range(0x10_0000, 0x1000));
        assert!(is_valid_ram_range(0x7FFF_F000, 0x1000));

        assert!(!is_valid_ram_range(0x8000_0000, 0x1000));
    }
}

#[test]
fn test_multiple_usable_regions_and_reserved_hole() {
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    reset_pmm_stats();
    unsafe {
        REGIONS[0] = UsableRegion {
            start: 0x1000_0000,
            end: 0x2000_0000,
            current: 0x1000_0000,
        };
        REGIONS[1] = UsableRegion {
            start: 0x3000_0000,
            end: 0x8000_0000,
            current: 0x3000_0000,
        };
        REGION_COUNT = 2;
        TOTAL_USABLE_RAM = 0x6000_0000;
        MAX_PHYS_ADDR = 0x8000_0000;

        assert!(is_valid_ram_range(0x1000_0000, 0x1000_0000));
        assert!(is_valid_ram_range(0x3000_0000, 0x4000_0000));

        assert!(!is_valid_ram_range(0x2000_0000, 0x1000));
        assert!(!is_valid_ram_range(0x2800_0000, 0x1000));

        assert!(!is_valid_ram_range(0x1800_0000, 0x2000_0000));
    }
}

#[test]
fn test_total_memory_vs_max_physical_address() {
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    reset_pmm_stats();
    unsafe {
        REGIONS[0] = UsableRegion {
            start: 0x1000_0000,
            end: 0x2000_0000,
            current: 0x1000_0000,
        };
        REGIONS[1] = UsableRegion {
            start: 0x3000_0000,
            end: 0x5000_0000,
            current: 0x3000_0000,
        };
        REGION_COUNT = 2;
        TOTAL_USABLE_RAM = 0x3000_0000;
        MAX_PHYS_ADDR = 0x5000_0000;

        assert_eq!(total_memory(), 0x3000_0000);
        assert_eq!(total_usable_memory(), 0x3000_0000);
        assert_eq!(max_physical_address(), 0x5000_0000);
    }
}

#[test]
fn test_1gib_boundary_and_exact_reclaim() {
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    reset_pmm_stats();
    set_test_ram_region(0x4000_0000, 0x8000_0000);

    let frame_1gb = 0x4000_0000u64;
    let count_1gb = 512 * 512;

    assert!(is_valid_ram_range(frame_1gb, 0x4000_0000));
    assert!(!is_valid_ram_range(frame_1gb, 0x4000_1000));

    assert_eq!(get_freed_frame_count(), 0);
    let ok = free_contiguous_frames(frame_1gb, count_1gb);
    assert!(ok);
    assert_eq!(get_freed_frame_count(), 262_144);
}

#[test]
fn test_partial_overlap_free_atomic_rejected() {
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    reset_pmm_stats();
    set_test_ram_region(0x20_0000, 0x20_6000);

    let frame_a = 0x20_0000u64;
    let frame_c = 0x20_2000u64;
    let frame_e = 0x20_4000u64;
    let frame_f = 0x20_5000u64;

    let res_ad = free_contiguous_frames(frame_a, 4);
    assert!(res_ad);
    assert_eq!(get_freed_frame_count(), 4);
    assert!(!is_frame_allocated(frame_a));
    assert!(!is_frame_allocated(frame_c));
    assert!(is_frame_allocated(frame_e));
    assert!(is_frame_allocated(frame_f));

    let res_cf = free_contiguous_frames(frame_c, 4);
    assert!(!res_cf);
    assert_eq!(get_freed_frame_count(), 4);
    assert!(is_frame_allocated(frame_e));
    assert!(is_frame_allocated(frame_f));

    let res_ef = free_contiguous_frames(frame_e, 2);
    assert!(res_ef);
    assert_eq!(get_freed_frame_count(), 6);
}

#[test]
fn test_double_free_middle_of_free_list_rejected() {
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    reset_pmm_stats();
    set_test_ram_region(0x20_0000, 0x20_3000);

    let frame_a = 0x20_0000u64;
    let frame_b = 0x20_1000u64;
    let frame_c = 0x20_2000u64;

    assert!(free_frame(frame_b));
    assert!(free_frame(frame_c));
    assert!(free_frame(frame_a));
    assert_eq!(get_freed_frame_count(), 3);

    let res_double_b = free_frame(frame_b);
    assert!(!res_double_b);
    assert_eq!(get_freed_frame_count(), 3);
}

#[test]
fn test_double_free_multiple_free_list_nodes() {
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    reset_pmm_stats();
    set_test_ram_region(0x20_0000, 0x20_5000);

    let f0 = 0x20_0000u64;
    let f2 = 0x20_2000u64;
    let f4 = 0x20_4000u64;

    assert!(free_frame(f0));
    assert!(free_frame(f2));
    assert!(free_frame(f4));
    assert_eq!(get_freed_frame_count(), 3);

    assert!(!free_frame(f0));
    assert!(!free_frame(f2));
    assert!(!free_frame(f4));
    assert_eq!(get_freed_frame_count(), 3);

    assert!(!free_contiguous_frames(f0, 3));
    assert_eq!(get_freed_frame_count(), 3);
}

#[test]
fn test_double_free_contiguous_range_rejected() {
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    reset_pmm_stats();
    set_test_ram_region(0x20_0000, 0x40_0000);

    let start_frame = 0x20_0000u64;
    let count = 512;

    let res1 = free_contiguous_frames(start_frame, count);
    assert!(res1);
    assert_eq!(get_freed_frame_count(), 512);

    let res2 = free_contiguous_frames(start_frame, count);
    assert!(!res2);
    assert_eq!(get_freed_frame_count(), 512);
    assert_eq!(used_memory(), 0);
}

#[test]
fn test_ram_greater_than_4gib_rejected() {
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    reset_pmm_stats();

    assert!(!is_valid_ram_range(0x1_0000_0000, 0x1000));
    assert!(!is_valid_ram_range(0x2_0000_0000, 0x1000));
    assert!(!is_frame_allocated(0x1_0000_0000));
}

#[test]
fn test_double_free_frame_greater_than_4gib_rejected() {
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    reset_pmm_stats();

    let res = free_frame(0x1_0000_0000);
    assert!(!res);
    assert_eq!(get_freed_frame_count(), 0);
}

#[test]
fn test_contiguous_free_greater_than_4gib_rejected() {
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    reset_pmm_stats();

    let res = free_contiguous_frames(0x1_0000_0000, 512);
    assert!(!res);
    assert_eq!(get_freed_frame_count(), 0);
}

#[test]
fn test_region_crossing_4gib_boundary_capped_or_rejected() {
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    reset_pmm_stats();

    set_test_ram_region(0xE000_0000, 0x1_2000_0000);

    assert!(is_valid_ram_range(0xE000_0000, 0x2000_0000));

    assert!(!is_valid_ram_range(0xE000_0000, 0x2000_1000));
    assert!(!is_valid_ram_range(0x1_0000_0000, 0x1000));

    let ok = free_contiguous_frames(0xE000_0000, (0x2000_0000 / PAGE_SIZE) as usize);
    assert!(ok);
    assert_eq!(get_freed_frame_count(), (0x2000_0000 / PAGE_SIZE) as u64);

    assert!(!free_frame(0x1_0000_0000));
    assert!(!free_contiguous_frames(0x1_0000_0000, 512));
}

#[test]
fn test_free_contiguous_frames_malformed_safely_ignored() {
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    reset_pmm_stats();
    let prev_freed = get_freed_frame_count();

    let res_unaligned = free_contiguous_frames(0x100001, 512);
    assert!(!res_unaligned);
    assert_eq!(get_freed_frame_count(), prev_freed);

    let res_zero = free_contiguous_frames(0x200000, 0);
    assert!(!res_zero);
    assert_eq!(get_freed_frame_count(), prev_freed);

    let res_low = free_contiguous_frames(0x0, 512);
    assert!(!res_low);
    assert_eq!(get_freed_frame_count(), prev_freed);

    let res_oob = free_contiguous_frames(0xF000_0000_0000, 512);
    assert!(!res_oob);
    assert_eq!(get_freed_frame_count(), prev_freed);
}

#[test]
#[should_panic(expected = "Recursive PMM lock detected")]
fn test_recursive_pmm_lock_debug_detection() {
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    set_test_cpu_id(0);
    let _g1 = PmmGuard::lock();
    let _g2 = PmmGuard::lock();
}

#[test]
fn test_different_cpu_waiting_does_not_panic_recursive() {
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    set_test_cpu_id(0);
    let g1 = PmmGuard::lock();

    let handle = std::thread::spawn(move || {
        set_test_cpu_id(1);
        let cur_cpu = get_current_cpu_id();
        assert_eq!(cur_cpu, 1);
        let holder = PMM_HOLDER_CORE.load(Ordering::Relaxed);
        assert_eq!(holder, 0);
        assert!(PMM_LOCK.load(Ordering::Relaxed));
        assert_ne!(holder, cur_cpu);
        clear_test_cpu_id();
    });

    handle.join().unwrap();
    drop(g1);
    clear_test_cpu_id();
}

#[test]
fn test_verify_pmm_invariants_locked_no_deadlock() {
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    reset_pmm_stats();
    set_test_ram_region_empty(0x20_0000, 0x40_0000);

    let _guard = PmmGuard::lock();
    assert!(verify_pmm_invariants_locked().is_ok());
}

#[test]
fn test_pmm_invariants_verification() {
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    reset_pmm_stats();
    assert!(verify_pmm_invariants().is_ok());

    set_test_ram_region(0x20_0000, 0x40_0000);
    assert!(verify_pmm_invariants().is_ok());

    let res = free_contiguous_frames(0x20_0000, 256);
    assert!(res);
    assert!(verify_pmm_invariants().is_ok());

    let res2 = free_contiguous_frames(0x30_0000, 256);
    assert!(res2);
    assert!(verify_pmm_invariants().is_ok());
}

#[test]
fn test_concurrent_alloc_free_stress() {
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    reset_pmm_stats();
    set_test_ram_region_empty(0x20_0000, 0x820_0000);

    let num_threads = 8;
    let iters_per_thread = 50;
    let mut handles = std::vec::Vec::new();

    for i in 0..num_threads {
        let handle = std::thread::spawn(move || {
            set_test_cpu_id(i as isize);
            for _ in 0..iters_per_thread {
                let f1 = alloc_frame();
                let f2 = alloc_frame();
                if let Some(frame) = f1 {
                    assert!(is_frame_allocated(frame));
                    assert!(free_frame(frame));
                    assert!(!is_frame_allocated(frame));
                }
                if let Some(frame) = f2 {
                    assert!(is_frame_allocated(frame));
                    assert!(free_frame(frame));
                    assert!(!is_frame_allocated(frame));
                }
            }
            clear_test_cpu_id();
        });
        handles.push(handle);
    }

    for h in handles {
        h.join().unwrap();
    }

    assert!(verify_pmm_invariants().is_ok());
}

#[test]
fn test_concurrent_alloc_free_race() {
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    reset_pmm_stats();
    set_test_ram_region_empty(0x20_0000, 0x220_0000);

    let mut handles = std::vec::Vec::new();
    for i in 0..4 {
        let h = std::thread::spawn(move || {
            set_test_cpu_id(i as isize);
            for _ in 0..100 {
                let f = alloc_frame();
                if let Some(frame) = f {
                    let _ = free_frame(frame);
                }
            }
            clear_test_cpu_id();
        });
        handles.push(h);
    }

    for h in handles {
        h.join().unwrap();
    }

    assert!(verify_pmm_invariants().is_ok());
}

#[test]
fn test_smp_baremetal_multi_core_stress() {
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    reset_pmm_stats();
    set_test_ram_region_empty(0x20_0000, 0x420_0000);

    let num_cores = 4;
    let mut handles = std::vec::Vec::new();

    for core_id in 0..num_cores {
        let handle = std::thread::spawn(move || {
            set_test_cpu_id(core_id as isize);
            for _ in 0..50 {
                let f = alloc_frame();
                if let Some(frame) = f {
                    assert!(is_frame_allocated(frame));
                    assert!(free_frame(frame));
                }
            }
            clear_test_cpu_id();
        });
        handles.push(handle);
    }

    for h in handles {
        h.join().unwrap();
    }

    assert!(verify_pmm_invariants().is_ok());
}

#[test]
fn test_irq_disabled_entry_exit_preservation() {
    let guard_if_was_enabled = PmmGuard { irq_state: true };
    assert!(guard_if_was_enabled.irq_state);

    let guard_if_was_disabled = PmmGuard { irq_state: false };
    assert!(!guard_if_was_disabled.irq_state);
}
