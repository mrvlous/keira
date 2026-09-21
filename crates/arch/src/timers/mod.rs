// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Hardware timers: Programmable Interval Timer (PIT), High Precision Event Timer (HPET), and POSIX timers.

pub mod hpet;
pub mod pit;
pub mod posix;

pub use hpet::{
    delay_micros as hpet_delay_micros, delay_millis as hpet_delay_millis,
    delay_nanos as hpet_delay_nanos, get_elapsed_nanos as hpet_get_elapsed_nanos,
    get_info as hpet_get_info, init_at as init_hpet_at, is_initialized as hpet_is_initialized,
    read_counter as hpet_read_counter, ticks_to_nanos as hpet_ticks_to_nanos, HpetInfo,
    DEFAULT_HPET_BASE, HPET_BASE_ADDR, HPET_INITIALIZED,
};
pub use pit::{get_uptime_ms, pit_handler, set_frequency as set_pit_frequency, uptime_ms};
pub use posix::*;
