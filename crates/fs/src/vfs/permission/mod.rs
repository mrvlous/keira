// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! POSIX-like user context security and filesystem access control policies.

pub mod checker;
pub mod user;

pub use self::checker::check_access_permission;
pub use self::user::{get_vfs_user, set_vfs_user, CURRENT_VFS_USER, CURRENT_VFS_USER_LEN};
