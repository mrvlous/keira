// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Process identity, permissions, and credential descriptors.

pub mod creds;

pub use creds::{
    get_current_egid, get_current_euid, get_current_gid, get_current_uid, set_current_gid,
    set_current_uid,
};
