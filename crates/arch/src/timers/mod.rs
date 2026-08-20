// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Hardware timers: Programmable Interval Timer (PIT), High Precision Event Timer (HPET), and POSIX timers.

pub mod hpet;
pub mod pit;
pub mod posix;

pub use hpet::{is_initialized as hpet_is_initialized, HPET_BASE_ADDR, HPET_INITIALIZED};
pub use pit::{get_uptime_ms, pit_handler, set_frequency as set_pit_frequency, uptime_ms};
pub use posix::*;
