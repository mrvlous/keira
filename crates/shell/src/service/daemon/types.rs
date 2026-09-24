// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Service state, telemetry records, and constant definitions for background daemons.

pub const MAX_SERVICES: usize = 16;
pub const CONF_DIR: &str = "/config/sys";
pub const MAX_LOG_LINES: usize = 4;
pub const MAX_LOG_LEN: usize = 64;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ServiceState {
    Stopped,
    Running,
    Failed,
}

/// In-memory circular log entry for recent service events and telemetry.
#[derive(Copy, Clone)]
pub struct ServiceLogEntry {
    pub text: [u8; MAX_LOG_LEN],
    pub len: usize,
    pub timestamp_ms: u64,
}

impl ServiceLogEntry {
    pub const fn empty() -> Self {
        Self {
            text: [0u8; MAX_LOG_LEN],
            len: 0,
            timestamp_ms: 0,
        }
    }

    pub fn as_str(&self) -> &str {
        core::str::from_utf8(&self.text[..self.len]).unwrap_or("")
    }
}

/// Service control record describing configuration, lifecycle state, and counters.
#[derive(Copy, Clone)]
pub struct ServiceRecord {
    pub name: [u8; 16],
    pub name_len: usize,
    pub desc: [u8; 48],
    pub desc_len: usize,
    pub state: ServiceState,
    pub pid: u32,
    pub enabled: bool,
    pub auto_restart: bool,
    pub port: u16,
    pub interval_secs: u32,
    pub last_tick_ms: u64,
    pub start_time_ms: u64,
    pub cycles_count: u64,
    pub bytes_count: u64,
    pub conf_path: [u8; 32],
    pub conf_path_len: usize,
    pub logs: [ServiceLogEntry; MAX_LOG_LINES],
    pub log_head: usize,
}

impl ServiceRecord {
    pub const fn empty() -> Self {
        Self {
            name: [0u8; 16],
            name_len: 0,
            desc: [0u8; 48],
            desc_len: 0,
            state: ServiceState::Stopped,
            pid: 0,
            enabled: false,
            auto_restart: true,
            port: 0,
            interval_secs: 0,
            last_tick_ms: 0,
            start_time_ms: 0,
            cycles_count: 0,
            bytes_count: 0,
            conf_path: [0u8; 32],
            conf_path_len: 0,
            logs: [ServiceLogEntry::empty(); MAX_LOG_LINES],
            log_head: 0,
        }
    }

    pub fn name_str(&self) -> &str {
        core::str::from_utf8(&self.name[..self.name_len]).unwrap_or("unknown")
    }

    pub fn desc_str(&self) -> &str {
        core::str::from_utf8(&self.desc[..self.desc_len]).unwrap_or("")
    }

    pub fn conf_path_str(&self) -> &str {
        core::str::from_utf8(&self.conf_path[..self.conf_path_len]).unwrap_or("")
    }

    pub fn log_event(&mut self, text: &str, timestamp_ms: u64) {
        let idx = self.log_head % MAX_LOG_LINES;
        let tbytes = text.as_bytes();
        let len = tbytes.len().min(MAX_LOG_LEN);
        self.logs[idx].text[..len].copy_from_slice(&tbytes[..len]);
        self.logs[idx].len = len;
        self.logs[idx].timestamp_ms = timestamp_ms;
        self.log_head = self.log_head.wrapping_add(1);
    }
}
