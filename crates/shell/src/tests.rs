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
