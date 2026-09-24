// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Unit tests for system telemetry and management command suite.

use super::*;

#[test]
fn test_sys_power() {
    let mut args = "power --help".split_whitespace();
    args.next();
    power::run(&mut args);
}

#[test]
fn test_sys_reset() {
    let mut args = "reset --help".split_whitespace();
    args.next();
    reset::run(&mut args);
}

#[test]
fn test_sys_runtime() {
    let mut args = "runtime --help".split_whitespace();
    args.next();
    runtime::run(&mut args);
}

#[test]
fn test_sys_sync() {
    let mut args = "sync --help".split_whitespace();
    args.next();
    sync::run(&mut args);
}

#[test]
fn test_sys_env() {
    let mut args = "env --help".split_whitespace();
    args.next();
    env::run(&mut args);
}

#[test]
fn test_sys_service() {
    let mut args = "service --help".split_whitespace();
    args.next();
    service::run(&mut args);
}

#[test]
fn test_sys_syslog() {
    let mut args = "syslog --help".split_whitespace();
    args.next();
    syslog::run(&mut args);
}

#[test]
fn test_sys_watchpoint() {
    let mut args = "watchpoint --help".split_whitespace();
    args.next();
    watchpoint::run(&mut args);
}

#[test]
fn test_sys_cpu() {
    let mut args = "cpu --help".split_whitespace();
    args.next();
    cpu::run(&mut args);
}

#[test]
fn test_sys_hostname() {
    let mut args = "hostname --help".split_whitespace();
    args.next();
    hostname::run(&mut args);
}

#[test]
fn test_sys_memory() {
    let mut args = "memory --help".split_whitespace();
    args.next();
    memory::run(&mut args);
}

#[test]
fn test_sys_smp() {
    let mut args = "smp --help".split_whitespace();
    args.next();
    smp::run(&mut args);
}

#[test]
fn test_sys_system() {
    let mut args = "system --help".split_whitespace();
    args.next();
    system::run(&mut args);
}

#[test]
fn test_sys_time() {
    let mut args = "time --help".split_whitespace();
    args.next();
    time::run(&mut args);
}

#[test]
fn test_sys_unwind() {
    let mut args = "unwind --help".split_whitespace();
    args.next();
    unwind::run(&mut args);
}
