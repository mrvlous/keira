// SPDX-License-Identifier: GPL-2.0-only
/*
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra <mrvlous@proton.me>
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 */

//! Unit tests for process and task command suite.

use super::*;

#[test]
fn test_proc_commands_help_invocation() {
    let mut args = "run --help".split_whitespace();
    args.next();
    run::run(&mut args);
}

#[test]
fn test_pidinfo_command_invocation() {
    let mut args = "pidinfo --help".split_whitespace();
    args.next();
    pidinfo::run(&mut args);

    let mut args_empty = "".split_whitespace();
    pidinfo::run(&mut args_empty);

    let mut args_pid = "0".split_whitespace();
    pidinfo::run(&mut args_pid);

    let mut args_invalid = "999".split_whitespace();
    pidinfo::run(&mut args_invalid);
}
