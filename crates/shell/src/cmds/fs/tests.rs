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
    let mut args = "folder --help".split_whitespace();
    args.next();
    folder::run(&mut args);

    let mut args = "list --help".split_whitespace();
    args.next();
    list::run(&mut args);

    let mut args = "copy --help".split_whitespace();
    args.next();
    copy::run(&mut args);

    let mut args = "create --help".split_whitespace();
    args.next();
    create::run(&mut args);

    let mut args = "delete --help".split_whitespace();
    args.next();
    delete::run(&mut args);

    let mut args = "edit --help".split_whitespace();
    args.next();
    edit::run(&mut args);

    let mut args = "fileinfo --help".split_whitespace();
    args.next();
    fileinfo::run(&mut args);

    let mut args = "move --help".split_whitespace();
    args.next();
    r#move::run(&mut args);

    let mut args = "view --help".split_whitespace();
    args.next();
    view::run(&mut args);

    let mut args = "write --help".split_whitespace();
    args.next();
    write::run(&mut args);

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
