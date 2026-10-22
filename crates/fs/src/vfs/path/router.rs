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
    if resolved.starts_with('/') {
        crate::vfs::mount::lookup_mount(resolved)
    } else {
        if resolved == "proc" || resolved.starts_with("proc/") {
            let sub = resolved.strip_prefix("proc/").unwrap_or("");
            (sub, FilesystemType::Proc)
        } else if resolved == "dev" || resolved.starts_with("dev/") {
            let sub = resolved.strip_prefix("dev/").unwrap_or("");
            (sub, FilesystemType::Dev)
        } else if resolved == "initrd" || resolved.starts_with("initrd/") {
            let sub = resolved.strip_prefix("initrd/").unwrap_or("");
            (sub, FilesystemType::Initrd)
        } else {
            (resolved, FilesystemType::Fat)
        }
    }
}
