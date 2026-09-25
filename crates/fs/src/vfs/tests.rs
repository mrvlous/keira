// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Unit tests for Virtual File System (VFS) path routing.

use super::*;

#[test]
fn test_resolve_alias_path() {
    assert_eq!(resolve_alias_path("/dev/null"), "/system/dev/null");
    assert_eq!(resolve_alias_path("/dev/zero"), "/system/dev/zero");
    assert_eq!(resolve_alias_path("/dev/random"), "/system/dev/random");
    assert_eq!(resolve_alias_path("/dev/urandom"), "/system/dev/urandom");
    assert_eq!(resolve_alias_path("/dev/tty"), "/system/dev/tty");
    assert_eq!(resolve_alias_path("/dev/ptmx"), "/system/dev/ptmx");
    assert_eq!(resolve_alias_path("/dev"), "/system/dev");
    assert_eq!(resolve_alias_path("/proc"), "/system/proc");
    assert_eq!(resolve_alias_path("/system/bin/init"), "/system/bin/init");
}

#[test]
fn test_route_path() {
    let (p, fs) = route_path("/system/proc/uptime");
    assert_eq!(p, "uptime");
    assert_eq!(fs, FilesystemType::Proc);

    let (p, fs) = route_path("/proc/meminfo");
    assert_eq!(p, "meminfo");
    assert_eq!(fs, FilesystemType::Proc);

    let (p, fs) = route_path("/system/dev/null");
    assert_eq!(p, "null");
    assert_eq!(fs, FilesystemType::Dev);

    let (p, fs) = route_path("/dev/zero");
    assert_eq!(p, "zero");
    assert_eq!(fs, FilesystemType::Dev);

    let (p, fs) = route_path("/initrd/system/kernel.elf");
    assert_eq!(p, "system/kernel.elf");
    assert_eq!(fs, FilesystemType::Initrd);

    let (p, fs) = route_path("/data/test.txt");
    assert_eq!(p, "/data/test.txt");
    assert_eq!(fs, FilesystemType::Fat);
}
