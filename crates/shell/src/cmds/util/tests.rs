// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Unit tests for utility command suite.

use super::*;

#[test]
fn test_util_commands_help_invocation() {
    let mut args = "help --help".split_whitespace();
    args.next();
    help::run(&mut args);
}
