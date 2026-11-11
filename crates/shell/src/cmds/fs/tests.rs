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

//! Unit tests for filesystem command suite.

use super::*;

#[test]
fn test_fs_commands_help_invocation() {
    let mut args = "disk --help".split_whitespace();
    args.next();
    disk::run(&mut args);

    let mut dcache_args = "dcache --help".split_whitespace();
    dcache_args.next();
    dcache::run(&mut dcache_args);

    let mut dcache_run_args = "dcache".split_whitespace();
    dcache_run_args.next();
    dcache::run(&mut dcache_run_args);

    let mut icache_help_args = "icache --help".split_whitespace();
    icache_help_args.next();
    icache::run(&mut icache_help_args);

    let mut icache_clear_args = "icache -c".split_whitespace();
    icache_clear_args.next();
    icache::run(&mut icache_clear_args);

    let mut icache_flush_args = "icache -f".split_whitespace();
    icache_flush_args.next();
    icache::run(&mut icache_flush_args);

    let mut icache_run_args = "icache".split_whitespace();
    icache_run_args.next();
    icache::run(&mut icache_run_args);
}
