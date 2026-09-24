// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Unit tests for process and task command suite.

use super::*;

#[test]
fn test_proc_commands_help_invocation() {
    let mut args = "bg --help".split_whitespace();
    args.next();
    bg::run(&mut args);

    let mut args = "fg --help".split_whitespace();
    args.next();
    fg::run(&mut args);

    let mut args = "jobs --help".split_whitespace();
    args.next();
    jobs::run(&mut args);

    let mut args = "kill --help".split_whitespace();
    args.next();
    kill::run(&mut args);

    let mut args = "stop --help".split_whitespace();
    args.next();
    stop::run(&mut args);

    let mut args = "cgroups --help".split_whitespace();
    args.next();
    cgroups::run(&mut args);

    let mut args = "futex --help".split_whitespace();
    args.next();
    futex::run(&mut args);

    let mut args = "run --help".split_whitespace();
    args.next();
    run::run(&mut args);

    let mut args = "tasks --help".split_whitespace();
    args.next();
    tasks::run(&mut args);

    let mut args = "timer --help".split_whitespace();
    args.next();
    timer::run(&mut args);

    let mut args = "eventfd --help".split_whitespace();
    args.next();
    eventfd::run(&mut args);

    let mut args = "kcc --help".split_whitespace();
    args.next();
    kcc::run(&mut args);

    let mut args = "perf --help".split_whitespace();
    args.next();
    perf::run(&mut args);
}
