// SPDX-License-Identifier: GPL-2.0-only

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
