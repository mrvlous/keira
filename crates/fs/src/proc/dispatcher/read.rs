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
    read_cmdline, read_cpuinfo, read_ip_tables_names, read_ip_tables_rules, read_loadavg,
    read_meminfo, read_mounts, read_net_arp, read_net_dev, read_net_raw, read_net_route,
    read_net_tcp, read_net_udp, read_nf_conntrack_stat, read_partitions, read_randomize_va_space,
    read_slabinfo, read_sockstat, read_uptime, read_version, read_vmstat,
};
use crate::proc::task::hooks::CURRENT_PID_HOOK;
use crate::proc::task::{
    get_active_pids, read_task_cmdline, read_task_stat, read_task_statm, read_task_status,
    task_exists,
};

/// Queries whether a procfs pseudo-node represents a directory.
pub fn is_proc_dir(clean_node: &str) -> bool {
    let node = clean_node.trim_start_matches('/');
    if node.is_empty()
        || node == "self"
        || node == "net"
        || node == "net/stat"
        || node == "sys"
        || node == "sys/kernel"
    {
        return true;
    }
    if let Ok(pid) = node.parse::<usize>() {
        return task_exists(pid);
    }
    false
}

/// Queries whether a procfs pseudo-node exists.
pub fn exists(clean_node: &str) -> bool {
    let node = clean_node.trim_start_matches('/');
    if matches!(
        node,
        "" | "uptime"
            | "meminfo"
            | "slabinfo"
            | "vmstat"
            | "cpuinfo"
            | "version"
            | "loadavg"
            | "cmdline"
            | "mounts"
            | "partitions"
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
            | "net/ip_tables_names"
            | "net/stat"
            | "net/stat/nf_conntrack"
            | "net/nf_conntrack"
            | "net/iptables"
            | "net/ip_tables_rules"
            | "sys"
            | "sys/kernel"
            | "sys/kernel/randomize_va_space"
    ) {
        return true;
    }
    if node == "self/status"
        || node == "self/cmdline"
        || node == "self/mounts"
        || node == "self/stat"
        || node == "self/statm"
    {
        return true;
    }
    if let Ok(pid) = node.parse::<usize>() {
        return task_exists(pid);
    }
    if let Some((pid_str, sub)) = node.split_once('/') {
        if sub == "status" || sub == "cmdline" || sub == "mounts" || sub == "stat" || sub == "statm"
        {
            if pid_str == "self" {
                return true;
            }
            if let Ok(pid) = pid_str.parse::<usize>() {
                return task_exists(pid);
            }
        }
    }
    false
}

/// Reads synthesized telemetry content from a procfs pseudo-node.
pub fn read_proc_file(clean_node: &str, buf: &mut [u8]) -> Result<usize, &'static str> {
    let node = clean_node.trim_start_matches('/');

    match node {
        "" | "net" | "net/stat" | "sys" | "sys/kernel" => Ok(0),
        "uptime" => read_uptime(buf),
        "meminfo" => read_meminfo(buf),
        "slabinfo" => read_slabinfo(buf),
        "vmstat" => read_vmstat(buf),
        "cpuinfo" => read_cpuinfo(buf),
        "version" => read_version(buf),
        "loadavg" => read_loadavg(buf),
        "cmdline" => read_cmdline(buf),
        "mounts" | "self/mounts" => read_mounts(buf),
        "partitions" => read_partitions(buf),
        "net/dev" => read_net_dev(buf),
        "net/route" => read_net_route(buf),
        "net/arp" => read_net_arp(buf),
        "net/tcp" => read_net_tcp(buf),
        "net/udp" => read_net_udp(buf),
        "net/raw" => read_net_raw(buf),
        "net/sockstat" | "sockstat" => read_sockstat(buf),
        "net/ip_tables_names" => read_ip_tables_names(buf),
        "net/stat/nf_conntrack" | "net/nf_conntrack" => read_nf_conntrack_stat(buf),
        "net/iptables" | "net/ip_tables_rules" => read_ip_tables_rules(buf),
        "sys/kernel/randomize_va_space" => read_randomize_va_space(buf),
        _ => {
            let (target_pid, sub) = if node == "self/status" {
                let pid = unsafe { CURRENT_PID_HOOK.map(|f| f()).unwrap_or(0) };
                (pid, "status")
            } else if node == "self/cmdline" {
                let pid = unsafe { CURRENT_PID_HOOK.map(|f| f()).unwrap_or(0) };
                (pid, "cmdline")
            } else if node == "self/stat" {
                let pid = unsafe { CURRENT_PID_HOOK.map(|f| f()).unwrap_or(0) };
                (pid, "stat")
            } else if node == "self/statm" {
                let pid = unsafe { CURRENT_PID_HOOK.map(|f| f()).unwrap_or(0) };
                (pid, "statm")
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
                "stat" => read_task_stat(target_pid, buf),
                "statm" => read_task_statm(target_pid, buf),
                _ => Err("Unsupported proc subfile"),
            }
        }
    }
}

use crate::vfs::VirtualDirEntry;

/// Virtual directory entries representing pseudo-files and directories in /proc.
pub static PROC_ENTRIES: [VirtualDirEntry; 12] = [
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
        name: "slabinfo",
        d_type: 8,
        ino: 17,
    },
    VirtualDirEntry {
        name: "vmstat",
        d_type: 8,
        ino: 18,
    },
    VirtualDirEntry {
        name: "self",
        d_type: 4,
        ino: 19,
    },
    VirtualDirEntry {
        name: "sys",
        d_type: 4,
        ino: 20,
    },
];

/// Pre-allocated static string representations for numeric process identifiers (0..63).
const PID_STRS: [&str; 64] = [
    "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12", "13", "14", "15", "16",
    "17", "18", "19", "20", "21", "22", "23", "24", "25", "26", "27", "28", "29", "30", "31", "32",
    "33", "34", "35", "36", "37", "38", "39", "40", "41", "42", "43", "44", "45", "46", "47", "48",
    "49", "50", "51", "52", "53", "54", "55", "56", "57", "58", "59", "60", "61", "62", "63",
];

/// Standard virtual directory entries resident inside every /proc/[pid] directory.
pub static PID_DIR_ENTRIES: [VirtualDirEntry; 7] = [
    VirtualDirEntry {
        name: ".",
        d_type: 4,
        ino: 1,
    },
    VirtualDirEntry {
        name: "..",
        d_type: 4,
        ino: 10,
    },
    VirtualDirEntry {
        name: "status",
        d_type: 8,
        ino: 2,
    },
    VirtualDirEntry {
        name: "cmdline",
        d_type: 8,
        ino: 3,
    },
    VirtualDirEntry {
        name: "stat",
        d_type: 8,
        ino: 4,
    },
    VirtualDirEntry {
        name: "statm",
        d_type: 8,
        ino: 5,
    },
    VirtualDirEntry {
        name: "mounts",
        d_type: 8,
        ino: 6,
    },
];

/// Retrieves a virtual directory entry inside /proc/[pid] by index.
pub fn get_pid_dir_entry(index: usize) -> Option<VirtualDirEntry> {
    PID_DIR_ENTRIES.get(index).copied()
}

/// Retrieves a virtual ProcFS directory entry by index, including active dynamic PIDs.
pub fn get_proc_entry(index: usize) -> Option<VirtualDirEntry> {
    if let Some(entry) = PROC_ENTRIES.get(index).copied() {
        return Some(entry);
    }
    let pid_idx = index.checked_sub(PROC_ENTRIES.len())?;
    let mut active_pids = [0usize; 64];
    let count = get_active_pids(&mut active_pids);
    if pid_idx < count {
        let pid = active_pids[pid_idx];
        if pid < PID_STRS.len() {
            return Some(VirtualDirEntry {
                name: PID_STRS[pid],
                d_type: 4,
                ino: 100 + pid as u64,
            });
        }
    }
    None
}
