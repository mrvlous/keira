// SPDX-License-Identifier: GPL-2.0-only

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
