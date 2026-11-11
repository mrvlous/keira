// SPDX-License-Identifier: GPL-2.0-only

//! Virtual File System (VFS) type definitions and metadata primitives.

pub mod fs_type;
pub mod node;

pub use self::fs_type::FilesystemType;
pub use self::node::{FileStat, NodeKind, VirtualDirEntry};
