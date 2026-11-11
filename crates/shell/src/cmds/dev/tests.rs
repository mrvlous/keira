// SPDX-License-Identifier: GPL-2.0-only

//! Unit tests for dev command modules.

use super::*;

#[test]
fn test_dev_commands_help_invocation() {
    let mut args = "devices --help".split_whitespace();
    args.next();
    devices::run(&mut args);

    let mut args = "drivers --help".split_whitespace();
    args.next();
    drivers::run(&mut args);
}
