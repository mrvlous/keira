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
    assert_eq!(resolve_alias_path("/dev/null"), "/dev/null");
    assert_eq!(resolve_alias_path("/dev/zero"), "/dev/zero");
    assert_eq!(resolve_alias_path("/dev/random"), "/dev/random");
    assert_eq!(resolve_alias_path("/dev/urandom"), "/dev/urandom");
    assert_eq!(resolve_alias_path("/dev/tty"), "/dev/tty");
    assert_eq!(resolve_alias_path("/dev/ptmx"), "/dev/ptmx");
    assert_eq!(resolve_alias_path("/dev"), "/dev");
    assert_eq!(resolve_alias_path("/proc"), "/proc");
    assert_eq!(resolve_alias_path("/proc/uptime"), "/proc/uptime");
    assert_eq!(resolve_alias_path("/bin/init"), "/bin/init");
}

#[test]
fn test_route_path() {
    let (p, fs) = route_path("/proc/uptime");
    assert_eq!(p, "uptime");
    assert_eq!(fs, FilesystemType::Proc);

    let (p, fs) = route_path("/proc/meminfo");
    assert_eq!(p, "meminfo");
    assert_eq!(fs, FilesystemType::Proc);

    let (p, fs) = route_path("/dev/null");
    assert_eq!(p, "null");
    assert_eq!(fs, FilesystemType::Dev);

    let (p, fs) = route_path("/dev/zero");
    assert_eq!(p, "zero");
    assert_eq!(fs, FilesystemType::Dev);

    let (p, fs) = route_path("/initrd/kernel.elf");
    assert_eq!(p, "kernel.elf");
    assert_eq!(fs, FilesystemType::Initrd);

    let (p, fs) = route_path("/tmp/test.txt");
    assert_eq!(p, "/tmp/test.txt");
    assert_eq!(fs, FilesystemType::Fat);
}
