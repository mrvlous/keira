// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Architecture-independent System Timer interfaces.
//!
//! Provides abstract periodic tick generation, system uptime accounting,
//! and calibrated busy-wait execution delays.

/// Generic Hardware System Timer trait.
pub trait Timer {
    /// Initializes the hardware timer with the requested periodic tick rate in Hertz.
    fn init(&mut self, frequency_hz: u32);

    /// Retrieves the total monotonic timer tick count accumulated since initialization.
    fn ticks(&self) -> u64;

    /// Retrieves system uptime elapsed in milliseconds.
    fn uptime_ms(&self) -> u64;

    /// Executes a calibrated busy wait for the specified duration in milliseconds.
    fn sleep_ms(&self, ms: u64);
}
