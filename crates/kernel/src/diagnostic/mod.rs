// SPDX-License-Identifier: GPL-2.0-only

//! Diagnostic logging, telemetry and panic handling subsystem.

#[cfg(target_os = "none")]
pub mod panic;

#[cfg(test)]
mod tests;
