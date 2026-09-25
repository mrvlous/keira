// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! System configuration, telemetry, services, and power shell commands.

pub mod control;
pub mod daemon;
pub mod info;

#[cfg(test)]
mod tests;

pub use control::{power, reset, runtime, sync};
pub use daemon::{syslog, watchpoint};
pub use info::{cpu, hostname, memory, smp, system, time, unwind};
