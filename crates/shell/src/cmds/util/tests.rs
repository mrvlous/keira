// SPDX-License-Identifier: GPL-2.0-only

//! Unit tests for utility command suite.

use super::*;

#[test]
fn test_util_commands_help_invocation() {
    let mut args = "help --help".split_whitespace();
    args.next();
    help::run(&mut args);
}
