// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Keira Service Controller (`ksvc`) & Background Daemon Management Subsystem.
//!
//! Manages native background services, daemon lifecycles, and configuration files (`.conf`)
//! stored in the canonical `/config/sys/` directory hierarchy.

pub mod config;
pub mod daemon;

#[cfg(test)]
mod tests;

pub use config::{parse_conf_into_record, parse_u32};
pub use daemon::{
    auto_start_enabled_services, enable_service, init, reload_service, reload_service_conf,
    reset_service_stats, restart_service, start_service, start_service_by_idx, stop_service,
    tick_all, ServiceLogEntry, ServiceRecord, ServiceState, CONF_DIR, MAX_LOG_LEN, MAX_LOG_LINES,
    MAX_SERVICES, SERVICES, SERVICE_COUNT,
};
