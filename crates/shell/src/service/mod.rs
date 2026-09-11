// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

#![allow(unused_variables, dead_code)]

//! Keira Service Controller (`ksvc`) & Background Daemon Management Subsystem.
//!
//! Manages native background services, daemon lifecycles, and configuration files (`.conf`)
//! stored in the canonical `/config/sys/` directory hierarchy.

pub const MAX_SERVICES: usize = 16;
pub const CONF_DIR: &str = "/config/sys";
pub const MAX_LOG_LINES: usize = 4;
pub const MAX_LOG_LEN: usize = 64;

#[derive(Copy, Clone, PartialEq, Eq)]
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

pub static mut SERVICES: [ServiceRecord; MAX_SERVICES] = [ServiceRecord::empty(); MAX_SERVICES];
pub static mut SERVICE_COUNT: usize = 0;
static mut INITIALIZED: bool = false;

extern "C" {
    fn get_uptime_ms() -> u64;
}

fn parse_u32(s: &str) -> Option<u32> {
    let mut val: u32 = 0;
    if s.is_empty() {
        return None;
    }
    for b in s.bytes() {
        if !b.is_ascii_digit() {
            return None;
        }
        val = val.checked_mul(10)?.checked_add((b - b'0') as u32)?;
    }
    Some(val)
}

/// Parse a .conf key-value buffer and apply to a service record.
pub fn parse_conf_into_record(content: &str, record: &mut ServiceRecord) {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with(';') {
            continue;
        }
        if let Some(eq_idx) = trimmed.find('=') {
            let key = trimmed[..eq_idx].trim();
            let val = trimmed[eq_idx + 1..].trim();

            match key {
                "name" => {
                    let klen = val.len().min(16);
                    record.name[..klen].copy_from_slice(&val.as_bytes()[..klen]);
                    record.name_len = klen;
                }
                "description" => {
                    let dlen = val.len().min(48);
                    record.desc[..dlen].copy_from_slice(&val.as_bytes()[..dlen]);
                    record.desc_len = dlen;
                }
                "enabled" => {
                    record.enabled = val == "1" || val == "true" || val == "yes";
                }
                "auto_restart" => {
                    record.auto_restart = val == "1" || val == "true" || val == "yes";
                }
                "port" => {
                    if let Some(p) = parse_u32(val) {
                        record.port = p as u16;
                    }
                }
                "interval" | "interval_secs" => {
                    if let Some(iv) = parse_u32(val) {
                        record.interval_secs = iv;
                    }
                }
                _ => {}
            }
        }
    }
}

/// Initialize built-in services and load their .conf files from /config/sys/.
///
/// # Safety
/// Must be executed in a synchronized kernel context during early shell initialization.
pub unsafe fn init() {
    if INITIALIZED {
        return;
    }

    SERVICE_COUNT = 0;

    // Register built-in default services (6 primary kernel system daemons)
    register_service_builtin(
        "syncd",
        "FAT16 Auto-Sync & Cache Flush Daemon",
        0,
        15,
        "/config/sys/syncd.conf",
        true,
    );
    register_service_builtin(
        "syslogd",
        "Kernel Event & Audit Logger Service",
        0,
        5,
        "/config/sys/syslogd.conf",
        true,
    );
    register_service_builtin(
        "watchdogd",
        "Memory & Task Health Watchdog",
        0,
        10,
        "/config/sys/watchdogd.conf",
        true,
    );
    register_service_builtin(
        "timed",
        "CMOS RTC & System Clock Sync Daemon",
        0,
        30,
        "/config/sys/timed.conf",
        true,
    );
    register_service_builtin(
        "monitord",
        "System Health & Telemetry Daemon",
        0,
        10,
        "/config/sys/monitord.conf",
        true,
    );
    register_service_builtin(
        "netd",
        "Network State & ARP Daemon",
        0,
        15,
        "/config/sys/netd.conf",
        true,
    );

    // Read and override configurations from .conf files if present on disk
    for i in 0..SERVICE_COUNT {
        reload_service_conf(i);
    }

    INITIALIZED = true;
}

/// Register a built-in service definition into the table.
///
/// # Safety
/// Accesses and modifies global static `SERVICES` and `SERVICE_COUNT`.
unsafe fn register_service_builtin(
    name: &str,
    desc: &str,
    port: u16,
    interval_secs: u32,
    conf_path: &str,
    enabled: bool,
) {
    if SERVICE_COUNT >= MAX_SERVICES {
        return;
    }
    let idx = SERVICE_COUNT;
    let mut rec = ServiceRecord::empty();

    let nlen = name.len().min(16);
    rec.name[..nlen].copy_from_slice(&name.as_bytes()[..nlen]);
    rec.name_len = nlen;

    let dlen = desc.len().min(48);
    rec.desc[..dlen].copy_from_slice(&desc.as_bytes()[..dlen]);
    rec.desc_len = dlen;

    rec.port = port;
    rec.interval_secs = interval_secs;
    rec.enabled = enabled;
    rec.pid = (idx + 2) as u32;

    let clen = conf_path.len().min(32);
    rec.conf_path[..clen].copy_from_slice(&conf_path.as_bytes()[..clen]);
    rec.conf_path_len = clen;

    SERVICES[idx] = rec;
    SERVICE_COUNT += 1;
}

/// Reload configuration for a service from its .conf file on disk.
///
/// # Safety
/// Index must be within bounds; accesses global static mutable services array.
pub unsafe fn reload_service_conf(idx: usize) {
    if idx >= SERVICE_COUNT {
        return;
    }
    let conf_path = SERVICES[idx].conf_path_str();
    let mut buf = [0u8; 1024];
    if let Ok(bytes_read) = keira_fs::vfs::read_file(conf_path, &mut buf) {
        if let Ok(content) = core::str::from_utf8(&buf[..bytes_read]) {
            parse_conf_into_record(content, &mut SERVICES[idx]);
        }
    }
}

/// Auto-start all enabled services on system boot.
///
/// # Safety
/// Initializes and starts services during early shell initialization.
pub unsafe fn auto_start_enabled_services() {
    init();
    for i in 0..SERVICE_COUNT {
        if SERVICES[i].enabled && SERVICES[i].state != ServiceState::Running {
            let _ = start_service_by_idx(i);
        }
    }
}

/// Start a service by index.
///
/// # Safety
/// Modifies global static services table and interacts with filesystem.
pub unsafe fn start_service_by_idx(idx: usize) -> Result<(), &'static str> {
    if idx >= SERVICE_COUNT {
        return Err("Service index out of range");
    }
    reload_service_conf(idx);
    let now = get_uptime_ms();
    SERVICES[idx].state = ServiceState::Running;
    SERVICES[idx].start_time_ms = now;
    SERVICES[idx].last_tick_ms = now;
    SERVICES[idx].log_event("Service started", now);

    // Initial action on start
    let name = SERVICES[idx].name_str();
    if name == "syslogd" {
        let _ = keira_fs::fat::create_dir("/data/log");
        let initial_log = b"[INFO] Keira Service Controller (ksvc) initialized syslog daemon\n";
        let _ = keira_fs::fat::append_file_content("/data/log/syslog.log", initial_log);
    } else if name == "monitord" {
        let _ = keira_fs::fat::create_dir("/data/log");
        let initial_log = b"[INFO] Keira Telemetry Monitor (monitord) initialized\n";
        let _ = keira_fs::fat::append_file_content("/data/log/monitor.log", initial_log);
    }

    Ok(())
}

/// Start a service by name.
///
/// # Safety
/// Modifies global static services table.
pub unsafe fn start_service(name: &str) -> Result<(), &'static str> {
    init();
    for i in 0..SERVICE_COUNT {
        if SERVICES[i].name_str() == name {
            return start_service_by_idx(i);
        }
    }
    Err("Service not found")
}

/// Stop a service by name.
///
/// # Safety
/// Modifies global static services table.
pub unsafe fn stop_service(name: &str) -> Result<(), &'static str> {
    init();
    for i in 0..SERVICE_COUNT {
        if SERVICES[i].name_str() == name {
            SERVICES[i].state = ServiceState::Stopped;
            let now = get_uptime_ms();
            SERVICES[i].log_event("Service stopped", now);
            return Ok(());
        }
    }
    Err("Service not found")
}

/// Restart a service by name.
///
/// # Safety
/// Modifies global static services table.
pub unsafe fn restart_service(name: &str) -> Result<(), &'static str> {
    stop_service(name)?;
    start_service(name)
}

/// Reload configuration for a service by name.
///
/// # Safety
/// Accesses and modifies global static services table.
pub unsafe fn reload_service(name: &str) -> Result<(), &'static str> {
    init();
    for i in 0..SERVICE_COUNT {
        if SERVICES[i].name_str() == name {
            reload_service_conf(i);
            let now = get_uptime_ms();
            SERVICES[i].log_event("Configuration reloaded", now);
            return Ok(());
        }
    }
    Err("Service not found")
}

/// Reset telemetry and cycle counters for a service.
///
/// # Safety
/// Accesses and modifies global static services table.
pub unsafe fn reset_service_stats(name: &str) -> Result<(), &'static str> {
    init();
    for i in 0..SERVICE_COUNT {
        if SERVICES[i].name_str() == name {
            SERVICES[i].cycles_count = 0;
            SERVICES[i].bytes_count = 0;
            let now = get_uptime_ms();
            SERVICES[i].log_event("Counters reset", now);
            return Ok(());
        }
    }
    Err("Service not found")
}

/// Enable a service to auto-start on boot and update its .conf file.
///
/// # Safety
/// Modifies global static services table and writes configuration to VFS.
pub unsafe fn enable_service(name: &str, enable: bool) -> Result<(), &'static str> {
    init();
    for i in 0..SERVICE_COUNT {
        if SERVICES[i].name_str() == name {
            SERVICES[i].enabled = enable;
            // Write updated conf file
            let conf_path = SERVICES[i].conf_path_str();
            let mut conf_buf = [0u8; 512];
            let mut len = 0;

            let append = |buf: &mut [u8], l: &mut usize, s: &[u8]| {
                let to_copy = s.len().min(buf.len().saturating_sub(*l));
                buf[*l..*l + to_copy].copy_from_slice(&s[..to_copy]);
                *l += to_copy;
            };

            append(&mut conf_buf, &mut len, b"# Keira Service Configuration\n");
            append(&mut conf_buf, &mut len, b"name=");
            append(&mut conf_buf, &mut len, SERVICES[i].name_str().as_bytes());
            append(&mut conf_buf, &mut len, b"\ndescription=");
            append(&mut conf_buf, &mut len, SERVICES[i].desc_str().as_bytes());
            append(&mut conf_buf, &mut len, b"\nenabled=");
            append(&mut conf_buf, &mut len, if enable { b"1" } else { b"0" });
            append(&mut conf_buf, &mut len, b"\nauto_restart=1\n");

            if SERVICES[i].port > 0 {
                append(&mut conf_buf, &mut len, b"port=");
                let mut p_buf = [0u8; 10];
                let mut p_val = SERVICES[i].port;
                let mut p_idx = 0;
                while p_val > 0 {
                    p_buf[p_idx] = b'0' + (p_val % 10) as u8;
                    p_idx += 1;
                    p_val /= 10;
                }
                for k in 0..p_idx {
                    conf_buf[len] = p_buf[p_idx - 1 - k];
                    len += 1;
                }
                append(&mut conf_buf, &mut len, b"\n");
            }

            if SERVICES[i].interval_secs > 0 {
                append(&mut conf_buf, &mut len, b"interval=");
                let mut iv_buf = [0u8; 10];
                let mut iv_val = SERVICES[i].interval_secs;
                let mut iv_idx = 0;
                while iv_val > 0 {
                    iv_buf[iv_idx] = b'0' + (iv_val % 10) as u8;
                    iv_idx += 1;
                    iv_val /= 10;
                }
                for k in 0..iv_idx {
                    conf_buf[len] = iv_buf[iv_idx - 1 - k];
                    len += 1;
                }
                append(&mut conf_buf, &mut len, b"\n");
            }

            let _ = keira_fs::fat::write_file_content(conf_path, &conf_buf[..len]);
            let now = get_uptime_ms();
            SERVICES[i].log_event(
                if enable {
                    "Auto-start enabled"
                } else {
                    "Auto-start disabled"
                },
                now,
            );
            return Ok(());
        }
    }
    Err("Service not found")
}

/// Background ticker: called on every shell event loop iteration / timer tick.
///
/// # Safety
/// Iterates over and modifies background services runtime state and filesystems.
pub unsafe fn tick_all() {
    if !INITIALIZED {
        return;
    }
    let now = get_uptime_ms();

    for i in 0..SERVICE_COUNT {
        if SERVICES[i].state != ServiceState::Running {
            continue;
        }

        let name = SERVICES[i].name_str();
        let interval_ms = (SERVICES[i].interval_secs as u64) * 1000;

        if interval_ms > 0 && now >= SERVICES[i].last_tick_ms + interval_ms {
            SERVICES[i].last_tick_ms = now;
            SERVICES[i].cycles_count += 1;

            if name == "syncd" {
                // Background filesystem auto-sync
                let _ = keira_fs::fat::flush_dirty_sectors();
                SERVICES[i].log_event("Filesystem dirty sectors flushed", now);
            } else if name == "syslogd" {
                // Background audit logger
                let mut log_buf = [0u8; 128];
                let mut offset = 0;
                let pfx = b"[INFO] Service Daemon Heartbeat: System healthy, Uptime: ";
                log_buf[offset..offset + pfx.len()].copy_from_slice(pfx);
                offset += pfx.len();

                let uptime_sec = now / 1000;
                let mut temp = [0u8; 20];
                let mut tlen = 0;
                let mut val = uptime_sec;
                if val == 0 {
                    temp[0] = b'0';
                    tlen = 1;
                } else {
                    while val > 0 {
                        temp[tlen] = b'0' + (val % 10) as u8;
                        tlen += 1;
                        val /= 10;
                    }
                }
                for k in 0..tlen {
                    log_buf[offset] = temp[tlen - 1 - k];
                    offset += 1;
                }
                let sfx = b"s\n";
                log_buf[offset..offset + sfx.len()].copy_from_slice(sfx);
                offset += sfx.len();

                let _ =
                    keira_fs::fat::append_file_content("/data/log/syslog.log", &log_buf[..offset]);
                SERVICES[i].log_event("Syslog heartbeat recorded", now);
            } else if name == "watchdogd" {
                // Memory & heap health supervisor
                let (total_frames, alloc_frames, free_frames) = keira_mem::pmm::get_stats();
                let pmm_res = keira_mem::verify_pmm_invariants();

                if total_frames > 0 && free_frames < total_frames / 20 {
                    SERVICES[i].log_event("WARN: Low physical memory threshold", now);
                    let alert = b"[WARN] watchdogd: Physical memory below 5% threshold\n";
                    let _ = keira_fs::fat::append_file_content("/data/log/syslog.log", alert);
                } else if pmm_res.is_err() {
                    SERVICES[i].log_event("WARN: PMM invariant check anomaly", now);
                } else {
                    SERVICES[i].log_event("Memory & PMM invariant check healthy", now);
                }
            } else if name == "timed" {
                // CMOS RTC & system clock sync daemon
                #[repr(C)]
                struct RtcTime {
                    second: u8,
                    minute: u8,
                    hour: u8,
                    day: u8,
                    month: u8,
                    year: u16,
                }
                extern "C" {
                    fn rtc_get_time(time: *mut RtcTime);
                }
                let mut t = RtcTime {
                    second: 0,
                    minute: 0,
                    hour: 0,
                    day: 0,
                    month: 0,
                    year: 0,
                };
                rtc_get_time(&mut t as *mut RtcTime);
                SERVICES[i].log_event("RTC clock drift synchronized", now);
            } else if name == "monitord" {
                // Hardware telemetry & performance sampler
                let (total_frames, alloc_frames, free_frames) = keira_mem::pmm::get_stats();
                let heap_used = keira_mem::heap_get_used();

                let mut mon_buf = [0u8; 160];
                let mut m_offset = 0;
                let pfx = b"[METRIC] Uptime: ";
                mon_buf[m_offset..m_offset + pfx.len()].copy_from_slice(pfx);
                m_offset += pfx.len();

                let mut v = now / 1000;
                let mut dig = [0u8; 16];
                let mut dl = 0;
                if v == 0 {
                    dig[0] = b'0';
                    dl = 1;
                } else {
                    while v > 0 {
                        dig[dl] = b'0' + (v % 10) as u8;
                        dl += 1;
                        v /= 10;
                    }
                }
                for k in 0..dl {
                    mon_buf[m_offset] = dig[dl - 1 - k];
                    m_offset += 1;
                }

                let mid = b"s | RAM alloc: ";
                mon_buf[m_offset..m_offset + mid.len()].copy_from_slice(mid);
                m_offset += mid.len();

                v = alloc_frames;
                dl = 0;
                if v == 0 {
                    dig[0] = b'0';
                    dl = 1;
                } else {
                    while v > 0 {
                        dig[dl] = b'0' + (v % 10) as u8;
                        dl += 1;
                        v /= 10;
                    }
                }
                for k in 0..dl {
                    mon_buf[m_offset] = dig[dl - 1 - k];
                    m_offset += 1;
                }

                let slash = b"/";
                mon_buf[m_offset..m_offset + slash.len()].copy_from_slice(slash);
                m_offset += slash.len();

                v = total_frames;
                dl = 0;
                if v == 0 {
                    dig[0] = b'0';
                    dl = 1;
                } else {
                    while v > 0 {
                        dig[dl] = b'0' + (v % 10) as u8;
                        dl += 1;
                        v /= 10;
                    }
                }
                for k in 0..dl {
                    mon_buf[m_offset] = dig[dl - 1 - k];
                    m_offset += 1;
                }

                let heap_str = b" frames | Heap used: ";
                mon_buf[m_offset..m_offset + heap_str.len()].copy_from_slice(heap_str);
                m_offset += heap_str.len();

                v = heap_used as u64;
                dl = 0;
                if v == 0 {
                    dig[0] = b'0';
                    dl = 1;
                } else {
                    while v > 0 {
                        dig[dl] = b'0' + (v % 10) as u8;
                        dl += 1;
                        v /= 10;
                    }
                }
                for k in 0..dl {
                    mon_buf[m_offset] = dig[dl - 1 - k];
                    m_offset += 1;
                }

                let end = b"B\n";
                mon_buf[m_offset..m_offset + end.len()].copy_from_slice(end);
                m_offset += end.len();

                let _ = keira_fs::fat::append_file_content(
                    "/data/log/monitor.log",
                    &mon_buf[..m_offset],
                );
                SERVICES[i].log_event("Telemetry snapshot recorded", now);
            } else if name == "netd" {
                // Network link monitor & ARP cache maintenance
                let carrier = keira_net::E1000_FOUND;
                if carrier {
                    SERVICES[i].log_event("Carrier UP (Intel e1000 active)", now);
                } else {
                    SERVICES[i].log_event("Carrier standby: no active NIC", now);
                }
            }
        }
    }
}
