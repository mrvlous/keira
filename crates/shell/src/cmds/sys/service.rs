// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

#![allow(unused_variables, unused_unsafe)]

//! Implementation of the 'service' / 'ksvc' command to manage background daemons and service .conf configurations.

use crate::editor::kvi::editor_start;
use crate::service::{
    enable_service, reload_service, reset_service_stats, restart_service, start_service,
    stop_service, ServiceState, SERVICES, SERVICE_COUNT,
};
use keira_io::vga;

extern "C" {
    fn get_uptime_ms() -> u64;
}

pub fn run(parts: &mut core::str::SplitWhitespace) {
    let subcmd = parts.next();

    match subcmd {
        None | Some("list") => {
            unsafe {
                crate::service::init();
                vga::set_color(vga::Color::White, vga::Color::Black);
                vga::print_str("Keira Service Controller (ksvc) - Background Daemons\n");
                vga::print_str(
                    "  SERVICE       PID    STATE      PORT / INTERVAL    CONFIG PATH\n",
                );
                vga::print_str("  ------------  -----  ---------  -----------------  ------------------------------\n");

                for i in 0..SERVICE_COUNT {
                    let svc = &SERVICES[i];
                    vga::set_color(vga::Color::White, vga::Color::Black);
                    vga::print_str("  ");

                    // Name (14 cols)
                    let name = svc.name_str();
                    vga::print_str(name);
                    for _ in name.len()..14 {
                        vga::print_str(" ");
                    }

                    // PID (7 cols)
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    vga::print_u64(svc.pid as u64);
                    let mut p_len = 1;
                    let mut p_val = svc.pid;
                    while p_val >= 10 {
                        p_len += 1;
                        p_val /= 10;
                    }
                    for _ in p_len..7 {
                        vga::print_str(" ");
                    }

                    // State (11 cols)
                    match svc.state {
                        ServiceState::Running => {
                            vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                            vga::print_str("RUNNING    ");
                        }
                        ServiceState::Stopped => {
                            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                            vga::print_str("STOPPED    ");
                        }
                        ServiceState::Failed => {
                            vga::set_color(vga::Color::LightRed, vga::Color::Black);
                            vga::print_str("FAILED     ");
                        }
                    }

                    // Port / Interval (19 cols)
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    if svc.port > 0 {
                        vga::print_str("Port ");
                        vga::print_u64(svc.port as u64);
                        vga::print_str(" (TCP)");
                        let mut l = 11;
                        if svc.port < 100 {
                            l = 10;
                        }
                        for _ in l..19 {
                            vga::print_str(" ");
                        }
                    } else if svc.interval_secs > 0 {
                        vga::print_str("Interval ");
                        vga::print_u64(svc.interval_secs as u64);
                        vga::print_str("s");
                        let mut l = 11;
                        if svc.interval_secs < 10 {
                            l = 10;
                        }
                        for _ in l..19 {
                            vga::print_str(" ");
                        }
                    } else {
                        vga::print_str("Manual trigger     ");
                    }

                    // Config Path
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    vga::print_str(svc.conf_path_str());
                    vga::print_str("\n");
                }

                vga::print_str("\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            }
        }

        Some("start") => {
            let name = match parts.next() {
                Some(n) => n,
                None => {
                    unsafe {
                        vga::set_color(vga::Color::LightRed, vga::Color::Black);
                        vga::print_str(
                            "Error: Service name required. Usage: service start <name>\n",
                        );
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    }
                    return;
                }
            };
            unsafe {
                match start_service(name) {
                    Ok(_) => {
                        vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                        vga::print_str("[OK] Started background service: ");
                        vga::print_str(name);
                        vga::print_str("\n");
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    }
                    Err(e) => {
                        vga::set_color(vga::Color::LightRed, vga::Color::Black);
                        vga::print_str("Error starting service: ");
                        vga::print_str(e);
                        vga::print_str("\n");
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    }
                }
            }
        }

        Some("stop") => {
            let name = match parts.next() {
                Some(n) => n,
                None => {
                    unsafe {
                        vga::set_color(vga::Color::LightRed, vga::Color::Black);
                        vga::print_str(
                            "Error: Service name required. Usage: service stop <name>\n",
                        );
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    }
                    return;
                }
            };
            unsafe {
                match stop_service(name) {
                    Ok(_) => {
                        vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                        vga::print_str("[OK] Stopped service: ");
                        vga::print_str(name);
                        vga::print_str("\n");
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    }
                    Err(e) => {
                        vga::set_color(vga::Color::LightRed, vga::Color::Black);
                        vga::print_str("Error stopping service: ");
                        vga::print_str(e);
                        vga::print_str("\n");
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    }
                }
            }
        }

        Some("restart") => {
            let name = match parts.next() {
                Some(n) => n,
                None => {
                    unsafe {
                        vga::set_color(vga::Color::LightRed, vga::Color::Black);
                        vga::print_str(
                            "Error: Service name required. Usage: service restart <name>\n",
                        );
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    }
                    return;
                }
            };
            unsafe {
                match restart_service(name) {
                    Ok(_) => {
                        vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                        vga::print_str("[OK] Restarted service: ");
                        vga::print_str(name);
                        vga::print_str("\n");
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    }
                    Err(e) => {
                        vga::set_color(vga::Color::LightRed, vga::Color::Black);
                        vga::print_str("Error restarting service: ");
                        vga::print_str(e);
                        vga::print_str("\n");
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    }
                }
            }
        }

        Some("reload") => {
            let name = match parts.next() {
                Some(n) => n,
                None => {
                    unsafe {
                        vga::set_color(vga::Color::LightRed, vga::Color::Black);
                        vga::print_str(
                            "Error: Service name required. Usage: service reload <name>\n",
                        );
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    }
                    return;
                }
            };
            unsafe {
                match reload_service(name) {
                    Ok(_) => {
                        vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                        vga::print_str("[OK] Reloaded configuration for service: ");
                        vga::print_str(name);
                        vga::print_str("\n");
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    }
                    Err(e) => {
                        vga::set_color(vga::Color::LightRed, vga::Color::Black);
                        vga::print_str("Error reloading service: ");
                        vga::print_str(e);
                        vga::print_str("\n");
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    }
                }
            }
        }

        Some("reset") => {
            let name = match parts.next() {
                Some(n) => n,
                None => {
                    unsafe {
                        vga::set_color(vga::Color::LightRed, vga::Color::Black);
                        vga::print_str(
                            "Error: Service name required. Usage: service reset <name>\n",
                        );
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    }
                    return;
                }
            };
            unsafe {
                match reset_service_stats(name) {
                    Ok(_) => {
                        vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                        vga::print_str("[OK] Reset counters and telemetry for service: ");
                        vga::print_str(name);
                        vga::print_str("\n");
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    }
                    Err(e) => {
                        vga::set_color(vga::Color::LightRed, vga::Color::Black);
                        vga::print_str("Error resetting service: ");
                        vga::print_str(e);
                        vga::print_str("\n");
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    }
                }
            }
        }

        Some("status") => {
            let name = match parts.next() {
                Some(n) => n,
                None => {
                    unsafe {
                        vga::set_color(vga::Color::LightRed, vga::Color::Black);
                        vga::print_str(
                            "Error: Service name required. Usage: service status <name>\n",
                        );
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    }
                    return;
                }
            };
            unsafe {
                crate::service::init();
                let mut found = false;
                for i in 0..SERVICE_COUNT {
                    if SERVICES[i].name_str() == name {
                        found = true;
                        let svc = &SERVICES[i];
                        vga::set_color(vga::Color::White, vga::Color::Black);
                        vga::print_str("Service Status: ");
                        vga::print_str(svc.name_str());
                        vga::print_str("\n");

                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                        vga::print_str("  Description   : ");
                        vga::print_str(svc.desc_str());
                        vga::print_str("\n");

                        vga::print_str("  State         : ");
                        match svc.state {
                            ServiceState::Running => {
                                vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                                vga::print_str("RUNNING (Active)\n");
                            }
                            ServiceState::Stopped => {
                                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                                vga::print_str("STOPPED (Inactive)\n");
                            }
                            ServiceState::Failed => {
                                vga::set_color(vga::Color::LightRed, vga::Color::Black);
                                vga::print_str("FAILED\n");
                            }
                        }
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);

                        vga::print_str("  Process PID   : ");
                        vga::print_u64(svc.pid as u64);
                        vga::print_str("\n");

                        vga::print_str("  Boot AutoStart: ");
                        if svc.enabled {
                            vga::print_str("Enabled\n");
                        } else {
                            vga::print_str("Disabled\n");
                        }

                        vga::print_str("  Config File   : ");
                        vga::print_str(svc.conf_path_str());
                        vga::print_str("\n");

                        if svc.port > 0 {
                            vga::print_str("  Listening Port: TCP ");
                            vga::print_u64(svc.port as u64);
                            vga::print_str("\n");
                        }

                        if svc.interval_secs > 0 {
                            vga::print_str("  Cycle Interval: ");
                            vga::print_u64(svc.interval_secs as u64);
                            vga::print_str(" seconds\n");
                        }

                        vga::print_str("  Cycles / Ticks: ");
                        vga::print_u64(svc.cycles_count);
                        vga::print_str("\n");

                        if svc.state == ServiceState::Running {
                            let now = get_uptime_ms();
                            let uptime_sec = (now.saturating_sub(svc.start_time_ms)) / 1000;
                            vga::print_str("  Service Uptime: ");
                            vga::print_u64(uptime_sec);
                            vga::print_str(" seconds\n");
                        }

                        vga::print_str("  Recent Events :\n");
                        let mut has_log = false;
                        for k in 0..crate::service::MAX_LOG_LINES {
                            let entry = &svc.logs[k];
                            if entry.len > 0 {
                                has_log = true;
                                vga::print_str("    - [");
                                vga::print_u64(entry.timestamp_ms / 1000);
                                vga::print_str("s] ");
                                vga::print_str(entry.as_str());
                                vga::print_str("\n");
                            }
                        }
                        if !has_log {
                            vga::print_str("    (No events logged yet)\n");
                        }
                        break;
                    }
                }
                if !found {
                    vga::set_color(vga::Color::LightRed, vga::Color::Black);
                    vga::print_str("Error: Service '");
                    vga::print_str(name);
                    vga::print_str("' not found in service registry.\n");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                }
            }
        }

        Some("logs") => {
            let name = match parts.next() {
                Some(n) => n,
                None => {
                    unsafe {
                        vga::set_color(vga::Color::LightRed, vga::Color::Black);
                        vga::print_str(
                            "Error: Service name required. Usage: service logs <name>\n",
                        );
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    }
                    return;
                }
            };
            unsafe {
                crate::service::init();
                let mut found = false;
                for i in 0..SERVICE_COUNT {
                    if SERVICES[i].name_str() == name {
                        found = true;
                        let svc = &SERVICES[i];
                        vga::set_color(vga::Color::White, vga::Color::Black);
                        vga::print_str("Logs for Service: ");
                        vga::print_str(svc.name_str());
                        vga::print_str(" (PID ");
                        vga::print_u64(svc.pid as u64);
                        vga::print_str(")\n");
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);

                        // 1. In-Memory Ring Buffer Events
                        vga::print_str("--- In-Memory Event Ring Buffer ---\n");
                        let mut has_mem = false;
                        for k in 0..crate::service::MAX_LOG_LINES {
                            let entry = &svc.logs[k];
                            if entry.len > 0 {
                                has_mem = true;
                                vga::print_str("  [+");
                                vga::print_u64(entry.timestamp_ms / 1000);
                                vga::print_str("s] ");
                                vga::print_str(entry.as_str());
                                vga::print_str("\n");
                            }
                        }
                        if !has_mem {
                            vga::print_str("  (No recent in-memory log entries)\n");
                        }

                        // 2. On-Disk Log Files (syslog / monitor)
                        let disk_path = if name == "syslogd" {
                            Some("/data/log/syslog.log")
                        } else if name == "monitord" {
                            Some("/data/log/monitor.log")
                        } else {
                            None
                        };

                        if let Some(lpath) = disk_path {
                            vga::print_str("\n--- Persistent Disk Log (");
                            vga::print_str(lpath);
                            vga::print_str(") ---\n");
                            let mut buf = [0u8; 1024];
                            if let Ok(br) = keira_fs::vfs::read_file(lpath, &mut buf) {
                                if let Ok(s) = core::str::from_utf8(&buf[..br]) {
                                    let mut line_count: usize = 0;
                                    for _ in s.lines() {
                                        line_count += 1;
                                    }
                                    let skip = line_count.saturating_sub(6);
                                    let mut cur: usize = 0;
                                    for line in s.lines() {
                                        if cur >= skip {
                                            vga::print_str("  ");
                                            vga::print_str(line);
                                            vga::print_str("\n");
                                        }
                                        cur += 1;
                                    }
                                }
                            } else {
                                vga::print_str("  (Disk log file not generated yet)\n");
                            }
                        }
                        vga::print_str("\n");
                        break;
                    }
                }
                if !found {
                    vga::set_color(vga::Color::LightRed, vga::Color::Black);
                    vga::print_str("Error: Service '");
                    vga::print_str(name);
                    vga::print_str("' not found in service registry.\n");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                }
            }
        }

        Some("top") | Some("stats") => {
            unsafe {
                crate::service::init();
                let now = get_uptime_ms();
                vga::set_color(vga::Color::White, vga::Color::Black);
                vga::print_str("Keira Service Controller (ksvc) - Daemon Real-Time Telemetry\n");
                vga::print_str(
                    "  PID    SERVICE       STATE      CYCLES    INTERVAL   UPTIME     LAST EVENT\n",
                );
                vga::print_str("  -----  ------------  ---------  --------  ---------  ---------  -------------------------------\n");

                for i in 0..SERVICE_COUNT {
                    let svc = &SERVICES[i];
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    vga::print_str("  ");

                    // PID (7 cols)
                    vga::print_u64(svc.pid as u64);
                    let mut p_len = 1;
                    let mut p_val = svc.pid;
                    while p_val >= 10 {
                        p_len += 1;
                        p_val /= 10;
                    }
                    for _ in p_len..7 {
                        vga::print_str(" ");
                    }

                    // Name (14 cols)
                    vga::set_color(vga::Color::White, vga::Color::Black);
                    let name = svc.name_str();
                    vga::print_str(name);
                    for _ in name.len()..14 {
                        vga::print_str(" ");
                    }

                    // State (11 cols)
                    match svc.state {
                        ServiceState::Running => {
                            vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                            vga::print_str("RUNNING    ");
                        }
                        ServiceState::Stopped => {
                            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                            vga::print_str("STOPPED    ");
                        }
                        ServiceState::Failed => {
                            vga::set_color(vga::Color::LightRed, vga::Color::Black);
                            vga::print_str("FAILED     ");
                        }
                    }

                    // Cycles (10 cols)
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    vga::print_u64(svc.cycles_count);
                    let mut c_len = 1;
                    let mut c_val = svc.cycles_count;
                    while c_val >= 10 {
                        c_len += 1;
                        c_val /= 10;
                    }
                    for _ in c_len..10 {
                        vga::print_str(" ");
                    }

                    // Interval (11 cols)
                    if svc.port > 0 {
                        vga::print_str("Port ");
                        vga::print_u64(svc.port as u64);
                        let mut l = 5;
                        if svc.port >= 10 {
                            l += 1;
                        }
                        if svc.port >= 100 {
                            l += 1;
                        }
                        for _ in l..11 {
                            vga::print_str(" ");
                        }
                    } else if svc.interval_secs > 0 {
                        vga::print_u64(svc.interval_secs as u64);
                        vga::print_str("s");
                        let mut l = 2;
                        if svc.interval_secs >= 10 {
                            l += 1;
                        }
                        if svc.interval_secs >= 100 {
                            l += 1;
                        }
                        for _ in l..11 {
                            vga::print_str(" ");
                        }
                    } else {
                        vga::print_str("manual     ");
                    }

                    // Uptime (11 cols)
                    let uptime_sec = if svc.state == ServiceState::Running {
                        (now.saturating_sub(svc.start_time_ms)) / 1000
                    } else {
                        0
                    };
                    vga::print_u64(uptime_sec);
                    vga::print_str("s");
                    let mut u_len = 2;
                    let mut u_val = uptime_sec;
                    while u_val >= 10 {
                        u_len += 1;
                        u_val /= 10;
                    }
                    for _ in u_len..11 {
                        vga::print_str(" ");
                    }

                    // Last Event
                    let last_log = if svc.log_head > 0 {
                        let prev_idx = (svc.log_head - 1) % crate::service::MAX_LOG_LINES;
                        svc.logs[prev_idx].as_str()
                    } else {
                        "Registered"
                    };
                    vga::print_str(last_log);
                    vga::print_str("\n");
                }
                vga::print_str("\n");
            }
        }

        Some("enable") => {
            let name = match parts.next() {
                Some(n) => n,
                None => {
                    unsafe {
                        vga::set_color(vga::Color::LightRed, vga::Color::Black);
                        vga::print_str(
                            "Error: Service name required. Usage: service enable <name>\n",
                        );
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    }
                    return;
                }
            };
            unsafe {
                match enable_service(name, true) {
                    Ok(_) => {
                        vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                        vga::print_str("[OK] Enabled auto-start on boot for service: ");
                        vga::print_str(name);
                        vga::print_str("\n");
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    }
                    Err(e) => {
                        vga::set_color(vga::Color::LightRed, vga::Color::Black);
                        vga::print_str("Error enabling service: ");
                        vga::print_str(e);
                        vga::print_str("\n");
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    }
                }
            }
        }

        Some("disable") => {
            let name = match parts.next() {
                Some(n) => n,
                None => {
                    unsafe {
                        vga::set_color(vga::Color::LightRed, vga::Color::Black);
                        vga::print_str(
                            "Error: Service name required. Usage: service disable <name>\n",
                        );
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    }
                    return;
                }
            };
            unsafe {
                match enable_service(name, false) {
                    Ok(_) => {
                        vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                        vga::print_str("[OK] Disabled auto-start on boot for service: ");
                        vga::print_str(name);
                        vga::print_str("\n");
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    }
                    Err(e) => {
                        vga::set_color(vga::Color::LightRed, vga::Color::Black);
                        vga::print_str("Error disabling service: ");
                        vga::print_str(e);
                        vga::print_str("\n");
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    }
                }
            }
        }

        Some("edit") => {
            let name = match parts.next() {
                Some(n) => n,
                None => {
                    unsafe {
                        vga::set_color(vga::Color::LightRed, vga::Color::Black);
                        vga::print_str(
                            "Error: Service name required. Usage: service edit <name>\n",
                        );
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    }
                    return;
                }
            };
            unsafe {
                crate::service::init();
                let mut path_buf = [0u8; 64];
                let prefix = b"/config/sys/";
                let suffix = b".conf";
                let mut offset = 0;
                path_buf[offset..offset + prefix.len()].copy_from_slice(prefix);
                offset += prefix.len();
                let nbytes = name.as_bytes();
                let to_copy = nbytes.len().min(path_buf.len() - offset - suffix.len());
                path_buf[offset..offset + to_copy].copy_from_slice(&nbytes[..to_copy]);
                offset += to_copy;
                path_buf[offset..offset + suffix.len()].copy_from_slice(suffix);
                offset += suffix.len();

                if let Ok(path_str) = core::str::from_utf8(&path_buf[..offset]) {
                    let _ = editor_start(path_str);
                }
            }
        }

        Some("create") => {
            let name = match parts.next() {
                Some(n) => n,
                None => {
                    unsafe {
                        vga::set_color(vga::Color::LightRed, vga::Color::Black);
                        vga::print_str(
                            "Error: Service name required. Usage: service create <name>\n",
                        );
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    }
                    return;
                }
            };
            unsafe {
                let mut path_buf = [0u8; 64];
                let prefix = b"/config/sys/";
                let suffix = b".conf";
                let mut offset = 0;
                path_buf[offset..offset + prefix.len()].copy_from_slice(prefix);
                offset += prefix.len();
                let nbytes = name.as_bytes();
                let to_copy = nbytes.len().min(path_buf.len() - offset - suffix.len());
                path_buf[offset..offset + to_copy].copy_from_slice(&nbytes[..to_copy]);
                offset += to_copy;
                path_buf[offset..offset + suffix.len()].copy_from_slice(suffix);
                offset += suffix.len();

                if let Ok(path_str) = core::str::from_utf8(&path_buf[..offset]) {
                    let template = b"# Keira Service Configuration\nname=custom\ndescription=Custom User Background Service\nenabled=0\ninterval=10\nauto_restart=1\n";
                    let _ = keira_fs::fat::write_file_content(path_str, template);
                    vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                    vga::print_str("[OK] Created new service configuration template: ");
                    vga::print_str(path_str);
                    vga::print_str("\n");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                }
            }
        }

        Some("-h") | Some("--help") | Some("help") => unsafe {
            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("Usage: service [list|start|stop|restart|status|enable|disable|edit|create|logs|top|reload|reset] [name]\n\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            vga::print_str("Description:\n  Keira Service Controller (ksvc) to manage background daemons and .conf files.\n\n");
            vga::print_str("Commands:\n");
            vga::print_str("  service list            List all registered background services and their state\n");
            vga::print_str("  service start <name>    Start a background daemon service\n");
            vga::print_str("  service stop <name>     Stop a running background service\n");
            vga::print_str(
                "  service restart <name>  Restart a service and reload its configuration\n",
            );
            vga::print_str(
                "  service reload <name>   Hot-reload .conf file without restarting service\n",
            );
            vga::print_str(
                "  service status <name>   Display detailed telemetry and uptime for a service\n",
            );
            vga::print_str(
                "  service logs <name>     View recent in-memory and on-disk service log history\n",
            );
            vga::print_str(
                "  service top / stats     Display real-time cycle and performance telemetry table\n",
            );
            vga::print_str("  service enable <name>   Enable service auto-start on boot\n");
            vga::print_str("  service disable <name>  Disable service auto-start on boot\n");
            vga::print_str(
                "  service reset <name>    Reset cycle counters and error state for a service\n",
            );
            vga::print_str(
                "  service edit <name>     Open /config/sys/<name>.conf in GNU nano editor\n",
            );
            vga::print_str("  service create <name>   Scaffold a new service config in /config/sys/<name>.conf\n\n");
            vga::print_str("Configuration Directory:\n  /config/sys/*.conf\n");
        },

        Some(other) => unsafe {
            vga::set_color(vga::Color::LightRed, vga::Color::Black);
            vga::print_str("Unknown service sub-command: '");
            vga::print_str(other);
            vga::print_str("'. Type 'service --help' for usage.\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        },
    }
}
