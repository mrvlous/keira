// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Architecture-independent System Timer traits.

/// Generic Hardware System Timer trait.
pub trait Timer {
    /// Initialize hardware timer with target periodic tick rate in Hz.
    fn init(&mut self, frequency_hz: u32);

    /// Retrieve total timer tick count since initialization.
    fn ticks(&self) -> u64;

    /// Retrieve system uptime in milliseconds.
    fn uptime_ms(&self) -> u64;

    /// Busy wait for specified duration in milliseconds.
    fn sleep_ms(&self, ms: u64);
}
