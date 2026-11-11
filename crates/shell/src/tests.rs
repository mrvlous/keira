// SPDX-License-Identifier: GPL-2.0-only

//! Top-level integration tests for Keira shell subsystem.

use super::*;

#[test]
fn test_shell_initialization_and_logo() {
    print_logo();
}

#[test]
fn test_shell_cli_args_parsing() {
    let input = "-l -a --long target_file.txt";
    let mut parts = input.split_whitespace();
    let cli = CliArgs::parse(&mut parts);
    assert!(cli.has_flag('l', "long"));
    assert!(cli.has_flag('a', "all"));
    assert_eq!(cli.first_positional(), Some("target_file.txt"));
}

#[test]
fn test_shell_dispatch_command() {
    execute_command("help");
}
