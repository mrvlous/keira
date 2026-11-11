// SPDX-License-Identifier: GPL-2.0-only

//! Standard POSIX filesystem mount flag constants.

/// Mount filesystem read-only.
pub const MS_RDONLY: u32 = 1;

/// Ignore suid and sgid bits on binaries within this filesystem.
pub const MS_NOSUID: u32 = 2;

/// Disallow access to device special files on this filesystem.
pub const MS_NODEV: u32 = 4;

/// Disallow program execution on this filesystem.
pub const MS_NOEXEC: u32 = 8;

/// Writes are synced at once.
pub const MS_SYNCHRONOUS: u32 = 16;

/// Alter flags of an existing mount point.
pub const MS_REMOUNT: u32 = 32;

/// Allow mandatory locks on this filesystem.
pub const MS_MANDLOCK: u32 = 64;

/// Directory modifications are synchronous.
pub const MS_DIRSYNC: u32 = 128;

/// Do not update access times.
pub const MS_NOATIME: u32 = 1024;

/// Do not update directory access times.
pub const MS_NODIRATIME: u32 = 2048;

/// Create a bind mount reflecting an existing tree.
pub const MS_BIND: u32 = 4096;
