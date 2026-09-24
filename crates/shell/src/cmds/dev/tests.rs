// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Unit tests for dev command modules.

use super::*;

#[test]
fn test_dev_commands_help_invocation() {
    let mut args = "devices --help".split_whitespace();
    args.next();
    devices::run(&mut args);

    let mut args = "framebuffer --help".split_whitespace();
    args.next();
    framebuffer::run(&mut args);

    let mut args = "usb --help".split_whitespace();
    args.next();
    usb::run(&mut args);

    let mut args = "lvm --help".split_whitespace();
    args.next();
    lvm::run(&mut args);

    let mut args = "nvme --help".split_whitespace();
    args.next();
    nvme::run(&mut args);

    let mut args = "raid --help".split_whitespace();
    args.next();
    raid::run(&mut args);

    let mut args = "swap --help".split_whitespace();
    args.next();
    swap::run(&mut args);

    let mut args = "drivers --help".split_whitespace();
    args.next();
    drivers::run(&mut args);

    let mut args = "epoll --help".split_whitespace();
    args.next();
    epoll::run(&mut args);

    let mut args = "kvm --help".split_whitespace();
    args.next();
    kvm::run(&mut args);

    let mut args = "lkm --help".split_whitespace();
    args.next();
    lkm::run(&mut args);
}
