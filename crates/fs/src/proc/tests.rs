// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Unit tests for dynamic ProcFS pseudo-filesystem nodes.

use super::*;

#[test]
fn test_procfs_nodes_exist() {
    assert!(exists("uptime"));
    assert!(exists("meminfo"));
    assert!(exists("cpuinfo"));
    assert!(exists("version"));
    assert!(exists("loadavg"));
    assert!(exists("cmdline"));
    assert!(exists("self/status"));
    assert!(exists("self/cmdline"));
    assert!(exists("1/status"));
    assert!(exists("1/cmdline"));
    assert!(!exists("nonexistent"));
}

#[test]
fn test_read_version() {
    let mut buf = [0u8; 256];
    let n = read_proc_file("version", &mut buf).expect("read version failed");
    let s = core::str::from_utf8(&buf[..n]).expect("utf8 version");
    assert!(s.contains("Keira Kernel version 0.4.0"));
}

#[test]
fn test_read_cpuinfo() {
    let mut buf = [0u8; 512];
    let n = read_proc_file("cpuinfo", &mut buf).expect("read cpuinfo failed");
    let s = core::str::from_utf8(&buf[..n]).expect("utf8 cpuinfo");
    assert!(s.contains("processor\t: 0"));
}

#[test]
fn test_read_uptime() {
    let mut buf = [0u8; 64];
    let n = read_proc_file("uptime", &mut buf).expect("read uptime failed");
    let s = core::str::from_utf8(&buf[..n]).expect("utf8 uptime");
    assert!(s.contains("0.00"));
}

#[test]
fn test_read_loadavg_and_cmdline() {
    let mut buf = [0u8; 128];
    let n = read_proc_file("loadavg", &mut buf).expect("read loadavg failed");
    let s = core::str::from_utf8(&buf[..n]).expect("utf8 loadavg");
    assert!(s.contains("0.00 0.00 0.00"));

    let n = read_proc_file("cmdline", &mut buf).expect("read cmdline failed");
    let s = core::str::from_utf8(&buf[..n]).expect("utf8 cmdline");
    assert!(s.contains("console=tty0"));
}
