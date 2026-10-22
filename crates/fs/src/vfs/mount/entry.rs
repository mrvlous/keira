// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! In-kernel mount entry descriptor and formatters.

use super::flags::*;
use crate::vfs::types::FilesystemType;

/// Maximum length of a mount source device name in bytes.
pub const MAX_MOUNT_DEV: usize = 64;

/// Maximum length of a mount target directory path in bytes.
pub const MAX_MOUNT_PATH: usize = 128;

/// In-kernel filesystem mount entry descriptor.
#[derive(Clone, Copy)]
pub struct MountEntry {
    /// Active allocation marker.
    pub is_active: bool,
    /// Source device or pseudo-filesystem identifier.
    pub source: [u8; MAX_MOUNT_DEV],
    /// Length of source string in bytes.
    pub source_len: usize,
    /// Target mount point directory path.
    pub target: [u8; MAX_MOUNT_PATH],
    /// Length of target path in bytes.
    pub target_len: usize,
    /// Underlying filesystem type.
    pub fs_type: FilesystemType,
    /// Mount flags (MS_RDONLY, MS_NOSUID, MS_NODEV, MS_NOEXEC).
    pub flags: u32,
}

impl MountEntry {
    /// Creates an empty inactive mount entry.
    pub const fn empty() -> Self {
        Self {
            is_active: false,
            source: [0u8; MAX_MOUNT_DEV],
            source_len: 0,
            target: [0u8; MAX_MOUNT_PATH],
            target_len: 0,
            fs_type: FilesystemType::Fat,
            flags: 0,
        }
    }

    /// Returns source device identifier string.
    pub fn source_str(&self) -> &str {
        if !self.is_active || self.source_len == 0 {
            return "";
        }
        core::str::from_utf8(&self.source[..self.source_len]).unwrap_or("")
    }

    /// Returns target mount point directory path string.
    pub fn target_str(&self) -> &str {
        if !self.is_active || self.target_len == 0 {
            return "";
        }
        core::str::from_utf8(&self.target[..self.target_len]).unwrap_or("")
    }

    /// Formats mount option flags into comma-separated text representation.
    pub fn format_flags(&self, buf: &mut [u8]) -> usize {
        let mut offset = 0;

        macro_rules! append_str {
            ($s:expr) => {
                let bytes = $s.as_bytes();
                if offset + bytes.len() <= buf.len() {
                    buf[offset..offset + bytes.len()].copy_from_slice(bytes);
                    offset += bytes.len();
                }
            };
        }

        if (self.flags & MS_RDONLY) != 0 {
            append_str!("ro");
        } else {
            append_str!("rw");
        }

        if (self.flags & MS_NOSUID) != 0 {
            append_str!(",nosuid");
        }
        if (self.flags & MS_NODEV) != 0 {
            append_str!(",nodev");
        }
        if (self.flags & MS_NOEXEC) != 0 {
            append_str!(",noexec");
        }
        if (self.flags & MS_SYNCHRONOUS) != 0 {
            append_str!(",sync");
        }
        if (self.flags & MS_NOATIME) != 0 {
            append_str!(",noatime");
        } else {
            append_str!(",relatime");
        }

        offset
    }
}
