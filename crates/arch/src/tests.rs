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
