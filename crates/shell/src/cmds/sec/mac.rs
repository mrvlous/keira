// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

#![allow(unused_variables, unused_unsafe)]

//! Inspect and configure Mandatory Access Control (MAC) and Type Enforcement policies.

use keira_io::vga;
use keira_task::security::mac::{
    check_path_access, get_audit_log, get_mode, get_rules, get_stats, set_mode, MacMode,
    MAC_APPEND, MAC_EXEC, MAC_READ, MAC_WRITE,
};

pub fn run(parts: &mut core::str::SplitWhitespace) {
    let subcmd = parts.next();

    if let Some("-h") | Some("--help") = subcmd {
        print_help();
        return;
    }

    match subcmd {
        None | Some("status") => {
            print_status();
        }
        Some("enforce") => {
            set_mode(MacMode::Enforcing);
            vga::set_color(vga::Color::LightGreen, vga::Color::Black);
            vga::print_str("[OK] ");
            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("Mandatory Access Control mode set to ");
            vga::set_color(vga::Color::LightGreen, vga::Color::Black);
            vga::print_str("Enforcing.\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            vga::print_str("     Policy violations will be strictly denied with EACCES.\n");
        }
        Some("permissive") => {
            set_mode(MacMode::Permissive);
            vga::set_color(vga::Color::LightGreen, vga::Color::Black);
            vga::print_str("[OK] ");
            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("Mandatory Access Control mode set to ");
            vga::set_color(vga::Color::Yellow, vga::Color::Black);
            vga::print_str("Permissive.\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            vga::print_str(
                "     Policy violations will be audited but operations will be allowed.\n",
            );
        }
        Some("disable") => {
            set_mode(MacMode::Disabled);
            vga::set_color(vga::Color::LightGreen, vga::Color::Black);
            vga::print_str("[OK] ");
            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("Mandatory Access Control set to ");
            vga::set_color(vga::Color::LightCyan, vga::Color::Black);
            vga::print_str("Disabled.\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        }
        Some("rules") => {
            print_rules();
        }
        Some("audit") | Some("log") => {
            print_audit_log();
        }
        Some("test") => {
            run_mac_tests();
        }
        Some(other) => {
            print_err("Unknown subcommand. Run 'mac --help' for usage.");
        }
    }
}

fn print_status() {
    let mode = get_mode();
    let (checks, violations) = get_stats();
    let rules = get_rules();
    let active_rules = rules.iter().filter(|r| r.in_use).count();

    vga::set_color(vga::Color::White, vga::Color::Black);
    vga::print_str("Mandatory Access Control (MAC) Subsystem:\n");
    vga::print_str("  Policy Engine     : Type Enforcement Active\n");
    vga::print_str("  Operational Mode  : ");

    match mode {
        MacMode::Disabled => {
            vga::set_color(vga::Color::LightCyan, vga::Color::Black);
            vga::print_str("Disabled (All Access Permitted)\n");
        }
        MacMode::Permissive => {
            vga::set_color(vga::Color::Yellow, vga::Color::Black);
            vga::print_str("Permissive (Auditing Violations without Blocking)\n");
        }
        MacMode::Enforcing => {
            vga::set_color(vga::Color::LightGreen, vga::Color::Black);
            vga::print_str("Enforcing (Violations Denied with EACCES)\n");
        }
    }

    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    vga::print_str("  Active Policy Rules: ");
    vga::print_u64(active_rules as u64);
    vga::print_str("\n  Paths Evaluated   : ");
    vga::print_u64(checks);
    vga::print_str("\n  Policy Violations : ");
    if violations > 0 {
        vga::set_color(vga::Color::LightRed, vga::Color::Black);
    }
    vga::print_u64(violations);
    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    vga::print_str("\n  Security Domains  : kernel, system, user, guest, network\n");
}

fn print_rules() {
    let rules = get_rules();

    vga::set_color(vga::Color::White, vga::Color::Black);
    vga::print_str("Mandatory Access Control Type Enforcement Policy Matrix:\n");
    vga::set_color(vga::Color::Cyan, vga::Color::Black);
    vga::print_str("  #   DOMAIN   TARGET PATH PREFIX        PERMISSIONS\n");
    vga::print_str("  --  -------  ------------------------  -----------\n");

    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    let mut count = 0;
    for (idx, rule) in rules.iter().enumerate() {
        if rule.in_use {
            count += 1;
            vga::print_str("  ");
            vga::print_u64(idx as u64);
            vga::print_str(if idx < 10 { "   " } else { "  " });

            let d_str = rule.domain.as_str();
            vga::print_str(d_str);
            for _ in 0..(9usize.saturating_sub(d_str.len())) {
                vga::print_str(" ");
            }

            if let Ok(prefix) = core::str::from_utf8(&rule.path_prefix[..rule.prefix_len]) {
                vga::print_str(prefix);
                for _ in 0..(26usize.saturating_sub(prefix.len())) {
                    vga::print_str(" ");
                }
            } else {
                vga::print_str("/???                      ");
            }

            let m = rule.allowed_mask;
            if m == 0 {
                vga::set_color(vga::Color::LightRed, vga::Color::Black);
                vga::print_str("[DENIED / NO ACCESS]");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            } else {
                if (m & MAC_READ) != 0 {
                    vga::print_str("R ");
                }
                if (m & MAC_WRITE) != 0 {
                    vga::print_str("W ");
                }
                if (m & MAC_EXEC) != 0 {
                    vga::print_str("X ");
                }
                if (m & MAC_APPEND) != 0 {
                    vga::print_str("A ");
                }
            }
            vga::print_str("\n");
        }
    }

    if count == 0 {
        vga::print_str("  No policy rules currently loaded.\n");
    }
}

fn print_audit_log() {
    let log = get_audit_log();

    vga::set_color(vga::Color::White, vga::Color::Black);
    vga::print_str("MAC Security Audit Event Log:\n");
    vga::set_color(vga::Color::Cyan, vga::Color::Black);
    vga::print_str("  PID  DOMAIN   REQ  ACTION   TARGET PATH\n");
    vga::print_str("  ---  -------  ---  -------  ----------------------\n");

    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    let mut count = 0;
    for ev_opt in log.iter() {
        if let Some(ev) = ev_opt {
            count += 1;
            vga::print_str("  ");
            vga::print_u64(ev.pid);
            vga::print_str("    ");

            let d_str = ev.domain.as_str();
            vga::print_str(d_str);
            for _ in 0..(9usize.saturating_sub(d_str.len())) {
                vga::print_str(" ");
            }

            let mut req_buf = [b'-'; 4];
            if (ev.requested_mask & MAC_READ) != 0 {
                req_buf[0] = b'r';
            }
            if (ev.requested_mask & MAC_WRITE) != 0 {
                req_buf[1] = b'w';
            }
            if (ev.requested_mask & MAC_EXEC) != 0 {
                req_buf[2] = b'x';
            }
            if (ev.requested_mask & MAC_APPEND) != 0 {
                req_buf[3] = b'a';
            }
            if let Ok(s) = core::str::from_utf8(&req_buf) {
                vga::print_str(s);
            }
            vga::print_str(" ");

            if ev.allowed {
                vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                vga::print_str("ALLOW    ");
            } else {
                vga::set_color(vga::Color::LightRed, vga::Color::Black);
                vga::print_str("DENY     ");
            }
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);

            if let Ok(path) = core::str::from_utf8(&ev.path[..ev.path_len]) {
                vga::print_str(path);
            }
            vga::print_str("\n");
        }
    }

    if count == 0 {
        vga::print_str("  Audit log is currently empty.\n");
    }
}

fn run_mac_tests() {
    vga::set_color(vga::Color::White, vga::Color::Black);
    vga::print_str("Running Type Enforcement policy evaluation tests:\n");

    let prev_mode = get_mode();
    set_mode(MacMode::Enforcing);

    // Test 1: User reading /system/bin/ls
    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    vga::print_str("  [1/4] User (PID 2) READ on /system/bin/ls ... ");
    let t1 = check_path_access(2, "/system/bin/ls", MAC_READ);
    print_decision(t1, true);

    // Test 2: User writing to /config/sys/passwd
    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    vga::print_str("  [2/4] User (PID 2) WRITE on /config/sys/passwd ... ");
    let t2 = check_path_access(2, "/config/sys/passwd", MAC_WRITE);
    print_decision(t2, false);

    // Test 3: System (PID 1) WRITE on /config/sys/passwd
    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    vga::print_str("  [3/4] System (PID 1) WRITE on /config/sys/passwd ... ");
    let t3 = check_path_access(1, "/config/sys/passwd", MAC_WRITE);
    print_decision(t3, true);

    // Test 4: User EXEC on /users/admin/script.sh
    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    vga::print_str("  [4/4] User (PID 2) EXEC on /users/admin/script.sh ... ");
    let t4 = check_path_access(2, "/users/admin/script.sh", MAC_EXEC);
    print_decision(t4, true);

    set_mode(prev_mode);

    vga::set_color(vga::Color::White, vga::Color::Black);
    vga::print_str("Type Enforcement test run complete. Restored previous operational mode.\n");
}

fn print_decision(actual: bool, expected: bool) {
    if actual == expected {
        vga::set_color(vga::Color::LightGreen, vga::Color::Black);
        if actual {
            vga::print_str("PASSED (Allowed)\n");
        } else {
            vga::print_str("PASSED (Blocked as forbidden)\n");
        }
    } else {
        vga::set_color(vga::Color::LightRed, vga::Color::Black);
        vga::print_str("FAILED (Unexpected result)\n");
    }
    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
}

fn print_help() {
    vga::print_str("Usage: mac [status|enforce|permissive|disable|rules|audit|test]\n\n");
    vga::print_str("Description:\n");
    vga::print_str(
        "  Inspect and configure Mandatory Access Control (MAC) and Type Enforcement policies.\n\n",
    );
    vga::print_str("Subcommands:\n");
    vga::print_str("  status                         Show MAC operational mode, evaluated paths, and violations (default)\n");
    vga::print_str("  enforce                        Enable Enforcing mode (deny unauthorized operations with EACCES)\n");
    vga::print_str("  permissive                     Enable Permissive mode (audit violations without blocking)\n");
    vga::print_str("  disable                        Disable MAC policy checks completely\n");
    vga::print_str(
        "  rules                          Display loaded Type Enforcement security policy matrix\n",
    );
    vga::print_str("  audit, log                     Display security audit event log records\n");
    vga::print_str("  test                           Run Type Enforcement test suite across domains and paths\n");
    vga::print_str("  -h, --help                     Show this help message and exit\n");
}

fn print_err(msg: &str) {
    vga::set_color(vga::Color::LightRed, vga::Color::Black);
    vga::print_str("[ERROR] ");
    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    vga::print_str(msg);
    vga::print_str("\n");
}
