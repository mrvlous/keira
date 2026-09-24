// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Path router dispatching canonical paths to underlying filesystem providers.

use super::alias::resolve_alias_path;
use crate::vfs::types::FilesystemType;

/// Routes an absolute or relative path to its target filesystem type and relative sub-path.
pub fn route_path(path: &str) -> (&str, FilesystemType) {
    let resolved = resolve_alias_path(path);
    if let Some(rest) = resolved.strip_prefix("/system/proc/") {
        (rest, FilesystemType::Proc)
    } else if resolved == "/system/proc" {
        ("", FilesystemType::Proc)
    } else if let Some(rest) = resolved.strip_prefix("/proc/") {
        (rest, FilesystemType::Proc)
    } else if resolved == "/proc" {
        ("", FilesystemType::Proc)
    } else if let Some(rest) = resolved.strip_prefix("/system/dev/") {
        (rest, FilesystemType::Dev)
    } else if resolved == "/system/dev" {
        ("", FilesystemType::Dev)
    } else if let Some(rest) = resolved.strip_prefix("/initrd/") {
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
