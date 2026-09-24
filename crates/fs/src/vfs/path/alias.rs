// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Canonical path alias resolution adhering to Keira 6-directory standard.

/// Resolves standard POSIX path aliases into canonical Keira hierarchy locations.
///
/// Aliases resolved:
/// - `/dev/<node>` -> `/system/dev/<node>`
/// - `/dev` -> `/system/dev`
/// - `/proc` -> `/system/proc`
pub fn resolve_alias_path(path: &str) -> &str {
    if let Some(rest) = path.strip_prefix("/dev/") {
        match rest {
            "null" => "/system/dev/null",
            "zero" => "/system/dev/zero",
            "random" => "/system/dev/random",
            "urandom" => "/system/dev/urandom",
            "tty" => "/system/dev/tty",
            "ptmx" => "/system/dev/ptmx",
            _ => path,
        }
    } else if path == "/dev" {
        "/system/dev"
    } else if path == "/proc" {
        "/system/proc"
    } else {
        path
    }
}
