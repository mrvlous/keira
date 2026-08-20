// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Path normalization, aliases resolution (`/dev/` -> `/system/dev/`), and filesystem routing.

use super::types::FilesystemType;

/// Resolve path aliases to Keira native directory standard (`/dev/` -> `/system/dev/`).
pub fn resolve_alias_path(path: &str) -> &str {
    if path.starts_with("/dev/") {
        match path {
            "/dev/null" => "/system/dev/null",
            "/dev/zero" => "/system/dev/zero",
            "/dev/random" => "/system/dev/random",
            "/dev/tty" => "/system/dev/tty",
            _ => path,
        }
    } else {
        path
    }
}

/// Route an absolute or relative path to its target filesystem and clean path.
pub fn route_path(path: &str) -> (&str, FilesystemType) {
    let resolved = resolve_alias_path(path);
    if let Some(rest) = resolved.strip_prefix("/initrd/") {
        (rest, FilesystemType::Initrd)
    } else if resolved == "/initrd" {
        ("", FilesystemType::Initrd)
    } else if let Some(rest) = resolved.strip_prefix("initrd/") {
        (rest, FilesystemType::Initrd)
    } else if resolved == "initrd" {
        ("", FilesystemType::Initrd)
    } else {
        (resolved, FilesystemType::Fat)
    }
}
