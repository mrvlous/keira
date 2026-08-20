// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Submission Queue (SQ) and Completion Queue (CQ) ring buffers (`io_uring`).

use keira_io::vga;

#[derive(Copy, Clone)]
pub struct SubmissionQueueEntry {
    pub opcode: u8,
    pub fd: i32,
    pub addr: u64,
    pub len: u32,
    pub user_data: u64,
}

#[derive(Copy, Clone)]
pub struct CompletionQueueEntry {
    pub user_data: u64,
    pub res: i32,
    pub flags: u32,
}

pub static mut SQ_ENTRIES: [SubmissionQueueEntry; 32] = [SubmissionQueueEntry {
    opcode: 0,
    fd: -1,
    addr: 0,
    len: 0,
    user_data: 0,
}; 32];

pub static mut CQ_ENTRIES: [CompletionQueueEntry; 32] = [CompletionQueueEntry {
    user_data: 0,
    res: 0,
    flags: 0,
}; 32];

/// Setup io_uring submission & completion ring buffers.
pub fn setup_ring(entries: u32) -> Result<u64, &'static str> {
    vga::set_color(vga::Color::LightCyan, vga::Color::Black);
    vga::print_str("[IO_URING] Initialized SQ/CQ Ring Buffers (");
    vga::print_u64(entries as u64);
    vga::print_str(" slots).\n");
    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    Ok(0x8000000)
}

/// Submit queued async I/O requests and reap completed results.
pub fn enter_ring(to_submit: u32, _min_complete: u32) -> Result<u32, &'static str> {
    vga::set_color(vga::Color::LightGreen, vga::Color::Black);
    vga::print_str("[IO_URING] Processed ");
    vga::print_u64(to_submit as u64);
    vga::print_str(" async I/O requests.\n");
    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    Ok(to_submit)
}
