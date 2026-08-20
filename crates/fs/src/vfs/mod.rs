// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Virtual File System (VFS) abstractions, path routing, and access permissions.

pub mod ops;
pub mod path;
pub mod permissions;
pub mod types;

pub use ops::*;
pub use path::*;
pub use permissions::*;
pub use types::*;
