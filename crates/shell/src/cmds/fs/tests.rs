// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Unit tests for filesystem command suite.

use super::*;

#[test]
fn test_fs_commands_help_invocation() {
    let mut args = "list --help".split_whitespace();
    args.next();
    list::run(&mut args);

    let mut args = "fileinfo --help".split_whitespace();
    args.next();
    fileinfo::run(&mut args);

    let mut args = "disk --help".split_whitespace();
    args.next();
    disk::run(&mut args);

    let mut args = "drives --help".split_whitespace();
    args.next();
    drives::run(&mut args);

    let mut args = "ext4 --help".split_whitespace();
    args.next();
    ext4::run(&mut args);

    let mut args = "initrd --help".split_whitespace();
    args.next();
    initrd::run(&mut args);

    let mut args = "ramdisk --help".split_whitespace();
    args.next();
    ramdisk::run(&mut args);

    let mut args = "use --help".split_whitespace();
    args.next();
    r#use::run(&mut args);
}
