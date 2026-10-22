// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! In-memory volatile TmpFS scratch filesystem implementation.

use keira_core::sync::SpinMutex;

/// Maximum number of concurrent in-memory files in TmpFS.
pub const MAX_TMPFS_FILES: usize = 16;

/// Maximum file size supported in TmpFS (8 KiB per file).
pub const MAX_TMPFS_FILE_SIZE: usize = 8192;

/// Maximum length of a TmpFS filename.
pub const MAX_TMPFS_NAME: usize = 64;

/// In-memory file node descriptor for TmpFS.
#[derive(Clone, Copy)]
pub struct TmpfsNode {
    pub is_active: bool,
    pub is_dir: bool,
    pub name: [u8; MAX_TMPFS_NAME],
    pub name_len: usize,
    pub data: [u8; MAX_TMPFS_FILE_SIZE],
    pub size: usize,
}

impl TmpfsNode {
    pub const fn empty() -> Self {
        Self {
            is_active: false,
            is_dir: false,
            name: [0u8; MAX_TMPFS_NAME],
            name_len: 0,
            data: [0u8; MAX_TMPFS_FILE_SIZE],
            size: 0,
        }
    }

    pub fn name_str(&self) -> &str {
        if !self.is_active || self.name_len == 0 {
            return "";
        }
        core::str::from_utf8(&self.name[..self.name_len]).unwrap_or("")
    }
}

/// Global synchronized TmpFS node table.
static TMPFS_NODES: SpinMutex<[TmpfsNode; MAX_TMPFS_FILES]> =
    SpinMutex::new([TmpfsNode::empty(); MAX_TMPFS_FILES]);

/// Normalizes path relative to TmpFS root.
fn sanitize_name(name: &str) -> &str {
    let clean = name.trim_matches('/');
    if clean.is_empty() {
        "."
    } else {
        clean
    }
}

/// Creates a new in-memory file in TmpFS.
pub fn create_file(name: &str) -> Result<(), &'static str> {
    let clean = sanitize_name(name);
    let mut nodes = TMPFS_NODES.lock();

    for node in nodes.iter() {
        if node.is_active && node.name_str() == clean {
            return Err("File already exists");
        }
    }

    for node in nodes.iter_mut() {
        if !node.is_active {
            node.is_active = true;
            node.is_dir = false;
            node.size = 0;
            node.name_len = clean.len().min(MAX_TMPFS_NAME);
            node.name[..node.name_len].copy_from_slice(&clean.as_bytes()[..node.name_len]);
            return Ok(());
        }
    }

    Err("TmpFS out of directory space")
}

/// Creates a directory entry in TmpFS.
pub fn create_dir(name: &str) -> Result<(), &'static str> {
    let clean = sanitize_name(name);
    let mut nodes = TMPFS_NODES.lock();

    for node in nodes.iter() {
        if node.is_active && node.name_str() == clean {
            return Err("Directory already exists");
        }
    }

    for node in nodes.iter_mut() {
        if !node.is_active {
            node.is_active = true;
            node.is_dir = true;
            node.size = 0;
            node.name_len = clean.len().min(MAX_TMPFS_NAME);
            node.name[..node.name_len].copy_from_slice(&clean.as_bytes()[..node.name_len]);
            return Ok(());
        }
    }

    Err("TmpFS out of directory space")
}

/// Writes content into a TmpFS file.
pub fn write_file(name: &str, content: &[u8]) -> Result<usize, &'static str> {
    let clean = sanitize_name(name);
    let mut nodes = TMPFS_NODES.lock();

    for node in nodes.iter_mut() {
        if node.is_active && node.name_str() == clean {
            if node.is_dir {
                return Err("Is a directory");
            }
            let to_write = content.len().min(MAX_TMPFS_FILE_SIZE);
            node.data[..to_write].copy_from_slice(&content[..to_write]);
            node.size = to_write;
            return Ok(to_write);
        }
    }

    // If file doesn't exist, create and write
    for node in nodes.iter_mut() {
        if !node.is_active {
            node.is_active = true;
            node.is_dir = false;
            node.name_len = clean.len().min(MAX_TMPFS_NAME);
            node.name[..node.name_len].copy_from_slice(&clean.as_bytes()[..node.name_len]);
            let to_write = content.len().min(MAX_TMPFS_FILE_SIZE);
            node.data[..to_write].copy_from_slice(&content[..to_write]);
            node.size = to_write;
            return Ok(to_write);
        }
    }

    Err("TmpFS full")
}

/// Appends content to a TmpFS file.
pub fn append_file(name: &str, content: &[u8]) -> Result<usize, &'static str> {
    let clean = sanitize_name(name);
    let mut nodes = TMPFS_NODES.lock();

    for node in nodes.iter_mut() {
        if node.is_active && node.name_str() == clean {
            if node.is_dir {
                return Err("Is a directory");
            }
            let available = MAX_TMPFS_FILE_SIZE.saturating_sub(node.size);
            let to_write = content.len().min(available);
            node.data[node.size..node.size + to_write].copy_from_slice(&content[..to_write]);
            node.size += to_write;
            return Ok(to_write);
        }
    }

    Err("No such file in TmpFS")
}

/// Reads content from a TmpFS file into buffer.
pub fn read_file(name: &str, buf: &mut [u8]) -> Result<usize, &'static str> {
    let clean = sanitize_name(name);
    let nodes = TMPFS_NODES.lock();

    for node in nodes.iter() {
        if node.is_active && node.name_str() == clean {
            if node.is_dir {
                return Err("Is a directory");
            }
            let to_read = node.size.min(buf.len());
            buf[..to_read].copy_from_slice(&node.data[..to_read]);
            return Ok(to_read);
        }
    }

    Err("No such file in TmpFS")
}

/// Reads file content starting from a byte offset.
pub fn read_file_offset(name: &str, offset: u64, buf: &mut [u8]) -> Result<usize, &'static str> {
    let clean = sanitize_name(name);
    let nodes = TMPFS_NODES.lock();

    for node in nodes.iter() {
        if node.is_active && node.name_str() == clean {
            if node.is_dir {
                return Err("Is a directory");
            }
            let off = offset as usize;
            if off >= node.size {
                return Ok(0);
            }
            let available = node.size - off;
            let to_read = available.min(buf.len());
            buf[..to_read].copy_from_slice(&node.data[off..off + to_read]);
            return Ok(to_read);
        }
    }

    Err("No such file in TmpFS")
}

/// Returns the size in bytes of a TmpFS file.
pub fn get_file_size(name: &str) -> Result<usize, &'static str> {
    let clean = sanitize_name(name);
    let nodes = TMPFS_NODES.lock();

    for node in nodes.iter() {
        if node.is_active && node.name_str() == clean {
            return Ok(node.size);
        }
    }

    Err("No such file in TmpFS")
}

/// Checks whether a file or directory exists in TmpFS.
pub fn exists(name: &str) -> bool {
    let clean = sanitize_name(name);
    if clean == "." || clean.is_empty() {
        return true;
    }
    let nodes = TMPFS_NODES.lock();
    nodes.iter().any(|n| n.is_active && n.name_str() == clean)
}

/// Checks whether a path is a directory in TmpFS.
pub fn is_dir(name: &str) -> bool {
    let clean = sanitize_name(name);
    if clean == "." || clean.is_empty() {
        return true;
    }
    let nodes = TMPFS_NODES.lock();
    nodes
        .iter()
        .any(|n| n.is_active && n.name_str() == clean && n.is_dir)
}

/// Removes a file or directory from TmpFS.
pub fn remove_entry(name: &str) -> Result<(), &'static str> {
    let clean = sanitize_name(name);
    let mut nodes = TMPFS_NODES.lock();

    for node in nodes.iter_mut() {
        if node.is_active && node.name_str() == clean {
            node.is_active = false;
            return Ok(());
        }
    }

    Err("No such file in TmpFS")
}

/// Returns directory entry for TmpFS at `index` (0-indexed).
///
/// Returns `Some((name_bytes, name_len, d_type, ino))` where:
/// - d_type: 4 for directory and 8 for regular file.
/// - ino: synthetic inode number.
pub fn get_tmpfs_entry(index: usize) -> Option<([u8; MAX_TMPFS_NAME], usize, u8, u64)> {
    if index == 0 {
        let mut name = [0u8; MAX_TMPFS_NAME];
        name[0] = b'.';
        return Some((name, 1, 4, 1));
    }
    if index == 1 {
        let mut name = [0u8; MAX_TMPFS_NAME];
        name[0] = b'.';
        name[1] = b'.';
        return Some((name, 2, 4, 2));
    }

    let target_idx = index - 2;
    let nodes = TMPFS_NODES.lock();
    let mut current_idx = 0;
    for (i, node) in nodes.iter().enumerate() {
        if node.is_active {
            if current_idx == target_idx {
                let d_type = if node.is_dir { 4 } else { 8 };
                let ino = (i as u64) + 10;
                return Some((node.name, node.name_len, d_type, ino));
            }
            current_idx += 1;
        }
    }
    None
}

/// Clears all allocated nodes in the in-memory TmpFS storage.
pub fn clear_all() {
    let mut nodes = TMPFS_NODES.lock();
    for node in nodes.iter_mut() {
        *node = TmpfsNode::empty();
    }
}
