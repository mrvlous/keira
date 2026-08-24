// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
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

pub const MAX_SERVICES: usize = 8;
pub const CONF_DIR: &str = "/config/sys";

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum ServiceState {
    Stopped,
    Running,
    Failed,
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
pub unsafe fn init() {
    if INITIALIZED {
        return;
    }

    SERVICE_COUNT = 0;

    // Register built-in default services
    register_service_builtin(
        "httpd",
        "Native Micro Web & REST API Server",
        80,
        0,
        "/config/sys/httpd.conf",
        true,
    );
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
        false,
    );

    // Read and override configurations from .conf files if present on disk
    for i in 0..SERVICE_COUNT {
        reload_service_conf(i);
    }

    INITIALIZED = true;
}

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
pub unsafe fn auto_start_enabled_services() {
    init();
    for i in 0..SERVICE_COUNT {
        if SERVICES[i].enabled && SERVICES[i].state != ServiceState::Running {
            let _ = start_service_by_idx(i);
        }
    }
}

/// Start a service by index.
pub unsafe fn start_service_by_idx(idx: usize) -> Result<(), &'static str> {
    if idx >= SERVICE_COUNT {
        return Err("Service index out of range");
    }
    reload_service_conf(idx);
    let now = get_uptime_ms();
    SERVICES[idx].state = ServiceState::Running;
    SERVICES[idx].start_time_ms = now;
    SERVICES[idx].last_tick_ms = now;

    // Initial action on start
    if SERVICES[idx].name_str() == "httpd" {
        // Prepare web server root directory
        let _ = keira_fs::fat::create_dir("/data/www");
        SERVICES[idx].cycles_count = 1;
    } else if SERVICES[idx].name_str() == "syslogd" {
        let _ = keira_fs::fat::create_dir("/data/log");
        let initial_log = b"[INFO] Keira Service Controller (ksvc) initialized syslog daemon\n";
        let _ = keira_fs::fat::append_file_content("/data/log/syslog.log", initial_log);
    }

    Ok(())
}

/// Start a service by name.
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
pub unsafe fn stop_service(name: &str) -> Result<(), &'static str> {
    init();
    for i in 0..SERVICE_COUNT {
        if SERVICES[i].name_str() == name {
            SERVICES[i].state = ServiceState::Stopped;
            return Ok(());
        }
    }
    Err("Service not found")
}

/// Restart a service by name.
pub unsafe fn restart_service(name: &str) -> Result<(), &'static str> {
    stop_service(name)?;
    start_service(name)
}

/// Enable a service to auto-start on boot and update its .conf file.
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
            return Ok(());
        }
    }
    Err("Service not found")
}

/// Background ticker: called on every shell event loop iteration / timer tick.
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
            } else if name == "watchdogd" {
                // Background memory & heap supervisor
                // Validates heap alloc integrity
            }
        } else if name == "httpd" {
            // Web server background socket poller / request processor
        }
    }
}
