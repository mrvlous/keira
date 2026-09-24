// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Integration tests for top-level filesystem abstractions and virtual node providers.

use super::*;

#[test]
fn test_procfs_and_devfs_existence() {
    assert!(exists("/system/dev/null"));
    assert!(exists("/system/dev/zero"));
    assert!(exists("/system/dev/random"));
    assert!(exists("/system/dev/urandom"));
    assert!(exists("/system/dev/tty"));
    assert!(exists("/dev/null"));
    assert!(exists("/dev/zero"));

    assert!(exists("/system/proc/uptime"));
    assert!(exists("/system/proc/meminfo"));
    assert!(exists("/system/proc/cpuinfo"));
    assert!(exists("/system/proc/version"));
    assert!(exists("/system/proc/loadavg"));
    assert!(exists("/system/proc/self/status"));
    assert!(exists("/proc/uptime"));
}

#[test]
fn test_read_proc_version_and_cpuinfo() {
    let mut buf = [0u8; 512];
    let n = read_proc_file("version", &mut buf).expect("read version failed");
    let s = core::str::from_utf8(&buf[..n]).expect("utf8 version");
    assert!(s.contains("Keira Kernel version 0.4.0"));

    let n = read_proc_file("cpuinfo", &mut buf).expect("read cpuinfo failed");
    let s = core::str::from_utf8(&buf[..n]).expect("utf8 cpuinfo");
    assert!(s.contains("processor\t: 0"));
}

#[test]
fn test_dev_null_and_zero_io() {
    let mut buf = [0x55u8; 16];
    let n = unsafe { dev::char::read_dev_node("zero", &mut buf) }.expect("read zero");
    assert_eq!(n, 16);
    assert_eq!(buf, [0u8; 16]);

    let written = unsafe { dev::char::write_dev_node("null", &[1, 2, 3, 4]) }.expect("write null");
    assert_eq!(written, 4);
}
