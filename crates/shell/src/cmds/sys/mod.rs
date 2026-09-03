// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! System configuration, telemetry, services, and power shell commands.

pub mod cpu;
pub mod env;
pub mod hostname;
pub mod memory;
pub mod power;
pub mod reset;
pub mod runtime;
pub mod service;
pub mod smp;
pub mod sync;
pub mod syslog;
pub mod system;
pub mod time;
pub mod unwind;
pub mod watchpoint;
