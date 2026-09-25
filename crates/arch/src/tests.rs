// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Architecture subsystem integration tests.

use super::*;

#[test]
fn test_posix_timer_lifecycle() {
    unsafe {
        let (active0, _expirations) = timer::get_timer_stats();
        assert!(active0 >= 1);

        let id = timer::create_timer(timer::CLOCK_MONOTONIC, 50).expect("Create timer failed");
        assert!(id >= 2);

        let table = timer::get_timer_table();
        assert!(table
            .iter()
            .any(|t| t.active && t.timer_id == id && t.interval_ms == 50));

        timer::cancel_timer(id).expect("Cancel failed");
    }
}

#[test]
fn test_perf_telemetry_and_reset() {
    unsafe {
        let snap = perf::get_perf_telemetry();
        assert!(snap.pmu_enabled);
        assert_eq!(snap.tsc_hz, 2_400_000_000);

        perf::reset_perf_counters();
    }
}

#[test]
fn test_irq_telemetry_counters() {
    let before = irq_get_total();
    isr_handler(32);
    let after = irq_get_total();
    assert_eq!(after, before + 1);
    assert!(irq_get_counter(32) >= 1);
    assert_eq!(irq_get_counter(256), 0);
}

#[test]
fn test_smp_boot_barrier_synchronization() {
    smp_barrier_reset(1);
    assert_eq!(smp_barrier_count(), 1);
    let count = smp_barrier_arrive();
    assert_eq!(count, 2);
    assert_eq!(smp_barrier_count(), 2);
    smp_barrier_reset(1);
    assert_eq!(smp_barrier_count(), 1);
}
