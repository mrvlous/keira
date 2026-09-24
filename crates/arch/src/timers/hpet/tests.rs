// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Unit tests for HPET MMIO register initialization, capabilities probing, and math routines.

use super::*;
use keira_core::sync::mutex::SpinMutex;

static HPET_TEST_LOCK: SpinMutex<()> = SpinMutex::new(());

#[test]
fn test_hpet_init_and_capabilities() {
    let _guard = HPET_TEST_LOCK.lock();
    unsafe {
        HPET_INITIALIZED = false;
        MOCK_HPET_MEM = [0u8; 1024];
        let mock_gcap = (10_000_000u64 << 32)
            | (0x8086u64 << 16)
            | (1u64 << 15)
            | (1u64 << 13)
            | (2u64 << 8)
            | 1u64;
        write_reg64(HPET_REG_GCAP_ID, mock_gcap);
    }

    let info = init_at(0xFED0_0000).expect("HPET initialization failed");
    assert!(is_initialized());
    assert_eq!(info.base_address, 0xFED0_0000);
    assert_eq!(info.vendor_id, 0x8086);
    assert_eq!(info.revision_id, 1);
    assert_eq!(info.num_timers, 3);
    assert!(info.is_64bit);
    assert!(info.legacy_route_capable);
    assert_eq!(info.period_fs, 10_000_000);
    assert_eq!(info.frequency_hz, 100_000_000);

    let retrieved = get_info();
    assert_eq!(retrieved, Some(info));
}

#[test]
fn test_hpet_counter_monotonic_advance() {
    let _guard = HPET_TEST_LOCK.lock();
    let _ = init_at(0xFED0_0000);
    let c1 = read_counter();
    let c2 = read_counter();
    let c3 = read_counter();

    assert!(c2 > c1, "Counter must advance monotonically");
    assert!(c3 > c2, "Counter must advance monotonically");
}

#[test]
fn test_hpet_ticks_to_nanos_math() {
    let _guard = HPET_TEST_LOCK.lock();
    let _ = init_at(0xFED0_0000);
    let nanos100 = ticks_to_nanos(100);
    assert_eq!(nanos100, 1_000);

    let nanos1sec = ticks_to_nanos(100_000_000);
    assert_eq!(nanos1sec, 1_000_000_000);
}

#[test]
fn test_hpet_invalid_period_rejection() {
    let _guard = HPET_TEST_LOCK.lock();
    unsafe {
        HPET_INITIALIZED = false;
        MOCK_HPET_MEM = [0u8; 1024];
        let invalid_gcap = (0u64 << 32) | (0x8086u64 << 16) | 1u64;
        write_reg64(HPET_REG_GCAP_ID, invalid_gcap);
    }

    let res = init_at(0xFED0_0000);
    assert!(res.is_err());
}

#[test]
fn test_hpet_delay_routines() {
    let _guard = HPET_TEST_LOCK.lock();
    let _ = init_at(0xFED0_0000);
    delay_nanos(100);
    delay_micros(1);
}
