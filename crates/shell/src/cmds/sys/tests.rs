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
fn test_sys_sync() {
    let mut args = "sync --help".split_whitespace();
    args.next();
    sync::run(&mut args);
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
fn test_sys_unwind() {
    let mut args = "unwind --help".split_whitespace();
    args.next();
    unwind::run(&mut args);
}

#[test]
fn test_sys_wipe() {
    let mut args = "wipe --help".split_whitespace();
    args.next();
    wipe::run(&mut args);

    let mut args_empty = "".split_whitespace();
    wipe::run(&mut args_empty);
}

#[test]
fn test_sys_regs() {
    let mut args = "regs --help".split_whitespace();
    args.next();
    regs::run(&mut args);

    let mut args_empty = "".split_whitespace();
    regs::run(&mut args_empty);
}

#[test]
fn test_sys_irq() {
    let mut args = "irq --help".split_whitespace();
    args.next();
    irq::run(&mut args);

    let mut args_empty = "".split_whitespace();
    irq::run(&mut args_empty);
}

#[test]
fn test_sys_slabinfo() {
    let mut args_help = "slabinfo --help".split_whitespace();
    args_help.next();
    slabinfo::run(&mut args_help);

    let mut args_cpu = "slabinfo -c".split_whitespace();
    args_cpu.next();
    slabinfo::run(&mut args_cpu);

    let mut args_verbose = "slabinfo -v".split_whitespace();
    args_verbose.next();
    slabinfo::run(&mut args_verbose);

    let mut args_default = "".split_whitespace();
    slabinfo::run(&mut args_default);
}
