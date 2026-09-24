// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Unit tests for security and authorization command suite.

use super::*;

#[test]
fn test_sec_commands_help_invocation() {
    let mut args = "login --help".split_whitespace();
    args.next();
    login::run(&mut args);

    let mut args = "protect --help".split_whitespace();
    args.next();
    protect::run(&mut args);

    let mut args = "user --help".split_whitespace();
    args.next();
    user::run(&mut args);

    let mut args = "bpf --help".split_whitespace();
    args.next();
    bpf::run(&mut args);

    let mut args = "mac --help".split_whitespace();
    args.next();
    mac::run(&mut args);

    let mut args = "seccomp --help".split_whitespace();
    args.next();
    seccomp::run(&mut args);

    let mut args = "tpm --help".split_whitespace();
    args.next();
    tpm::run(&mut args);
}
