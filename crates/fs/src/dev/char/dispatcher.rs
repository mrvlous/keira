// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Character device node operations dispatcher.

use super::{null, random, tty, zero};

/// Reads bytes from a special character device node.
///
/// # Safety
///
/// Dispatches to device drivers that may access hardware buffers.
pub unsafe fn read_dev_node(node_name: &str, buf: &mut [u8]) -> Result<usize, &'static str> {
    match node_name {
        "null" => null::read(buf),
        "zero" => zero::read(buf),
        "random" | "urandom" => random::read(buf),
        "ptmx" => Ok(0),
        "tty" => tty::read(buf),
        _ => Err("Unknown device node"),
    }
}

/// Writes bytes to a special character device node.
///
/// # Safety
///
/// Dispatches to device drivers that may access hardware buffers.
pub unsafe fn write_dev_node(node_name: &str, buf: &[u8]) -> Result<usize, &'static str> {
    match node_name {
        "null" => null::write(buf),
        "zero" => zero::write(buf),
        "random" | "urandom" => random::write(buf),
        "ptmx" => Ok(buf.len()),
        "tty" => tty::write(buf),
        _ => Err("Unknown device node"),
    }
}

/// Queries whether a character device node exists dynamically.
pub fn exists(node_name: &str) -> bool {
    let clean = node_name.trim_start_matches('/');
    matches!(
        clean,
        "null" | "zero" | "random" | "urandom" | "tty" | "ptmx"
    )
}
