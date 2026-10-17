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
    read_cmdline, read_cpuinfo, read_loadavg, read_meminfo, read_net_arp, read_net_dev,
    read_net_raw, read_net_route, read_net_tcp, read_net_udp, read_sockstat, read_uptime,
    read_version,
};
use crate::proc::task::hooks::CURRENT_PID_HOOK;
use crate::proc::task::{read_task_cmdline, read_task_status};

/// Queries whether a procfs pseudo-node exists.
pub fn exists(clean_node: &str) -> bool {
    let node = clean_node.trim_start_matches('/');
    if matches!(
        node,
        "" | "uptime"
            | "meminfo"
            | "cpuinfo"
            | "version"
            | "loadavg"
            | "cmdline"
            | "self"
            | "net"
            | "net/dev"
            | "net/route"
            | "net/arp"
            | "net/tcp"
            | "net/udp"
            | "net/raw"
            | "net/sockstat"
            | "sockstat"
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
        "" | "net" => Ok(0),
        "uptime" => read_uptime(buf),
        "meminfo" => read_meminfo(buf),
        "cpuinfo" => read_cpuinfo(buf),
        "version" => read_version(buf),
        "loadavg" => read_loadavg(buf),
        "cmdline" => read_cmdline(buf),
        "net/dev" => read_net_dev(buf),
        "net/route" => read_net_route(buf),
        "net/arp" => read_net_arp(buf),
        "net/tcp" => read_net_tcp(buf),
        "net/udp" => read_net_udp(buf),
        "net/raw" => read_net_raw(buf),
        "net/sockstat" | "sockstat" => read_sockstat(buf),
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

use crate::vfs::VirtualDirEntry;

/// Virtual directory entries representing pseudo-files and directories in /proc.
pub static PROC_ENTRIES: [VirtualDirEntry; 9] = [
    VirtualDirEntry {
        name: ".",
        d_type: 4,
        ino: 10,
    },
    VirtualDirEntry {
        name: "..",
        d_type: 4,
        ino: 10,
    },
    VirtualDirEntry {
        name: "uptime",
        d_type: 8,
        ino: 11,
    },
    VirtualDirEntry {
        name: "meminfo",
        d_type: 8,
        ino: 12,
    },
    VirtualDirEntry {
        name: "cpuinfo",
        d_type: 8,
        ino: 13,
    },
    VirtualDirEntry {
        name: "version",
        d_type: 8,
        ino: 14,
    },
    VirtualDirEntry {
        name: "loadavg",
        d_type: 8,
        ino: 15,
    },
    VirtualDirEntry {
        name: "cmdline",
        d_type: 8,
        ino: 16,
    },
    VirtualDirEntry {
        name: "self",
        d_type: 4,
        ino: 17,
    },
];

/// Retrieves a virtual ProcFS directory entry by index.
pub fn get_proc_entry(index: usize) -> Option<VirtualDirEntry> {
    PROC_ENTRIES.get(index).copied()
}
