// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Unit tests for network and IPC command suite.

use super::*;

#[test]
fn test_net_commands_help_invocation() {
    let mut args = "network --help".split_whitespace();
    args.next();
    network::run(&mut args);

    let mut args = "firewall --help".split_whitespace();
    args.next();
    firewall::run(&mut args);

    let mut args = "mqueue --help".split_whitespace();
    args.next();
    mqueue::run(&mut args);
}

#[test]
fn test_network_command_default_and_stats() {
    let mut args = "network".split_whitespace();
    args.next();
    network::run(&mut args);

    let mut args = "network -s".split_whitespace();
    args.next();
    network::run(&mut args);

    let mut args = "network ping 127.0.0.1".split_whitespace();
    args.next();
    network::run(&mut args);
}
