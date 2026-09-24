// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Dispatcher routing read operations to specific ProcFS nodes.

use crate::proc::system::{
    read_cmdline, read_cpuinfo, read_loadavg, read_meminfo, read_uptime, read_version,
};
use crate::proc::task::hooks::CURRENT_PID_HOOK;
use crate::proc::task::{read_task_cmdline, read_task_status};

/// Queries whether a procfs pseudo-node exists.
pub fn exists(clean_node: &str) -> bool {
    let node = clean_node.trim_start_matches('/');
    if matches!(
        node,
        "" | "uptime" | "meminfo" | "cpuinfo" | "version" | "loadavg" | "cmdline" | "self"
    ) {
        return true;
    }
    if node == "self/status" || node == "self/cmdline" {
        return true;
    }
    if let Some((pid_str, sub)) = node.split_once('/') {
        if sub == "status" || sub == "cmdline" {
            if pid_str == "self" {
                return true;
            }
            if pid_str.parse::<usize>().is_ok() {
                return true;
            }
        }
    }
    false
}

/// Reads synthesized telemetry content from a procfs pseudo-node.
pub fn read_proc_file(clean_node: &str, buf: &mut [u8]) -> Result<usize, &'static str> {
    let node = clean_node.trim_start_matches('/');

    match node {
        "" => Ok(0),
        "uptime" => read_uptime(buf),
        "meminfo" => read_meminfo(buf),
        "cpuinfo" => read_cpuinfo(buf),
        "version" => read_version(buf),
        "loadavg" => read_loadavg(buf),
        "cmdline" => read_cmdline(buf),
        _ => {
            let (target_pid, sub) = if node == "self/status" {
                let pid = unsafe { CURRENT_PID_HOOK.map(|f| f()).unwrap_or(0) };
                (pid, "status")
            } else if node == "self/cmdline" {
                let pid = unsafe { CURRENT_PID_HOOK.map(|f| f()).unwrap_or(0) };
                (pid, "cmdline")
            } else if let Some((pid_str, sub)) = node.split_once('/') {
                let pid = if pid_str == "self" {
                    unsafe { CURRENT_PID_HOOK.map(|f| f()).unwrap_or(0) }
                } else {
                    pid_str
                        .parse::<usize>()
                        .map_err(|_| "Invalid PID in proc path")?
                };
                (pid, sub)
            } else {
                return Err("Unknown proc node");
            };

            match sub {
                "status" => read_task_status(target_pid, buf),
                "cmdline" => read_task_cmdline(target_pid, buf),
                _ => Err("Unsupported proc subfile"),
            }
        }
    }
}
