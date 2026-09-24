// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Shared memory segment structures and semaphore descriptors.

pub mod types;

pub use super::manager::*;
pub use types::{
    Semaphore, ShmSegment, SEM_CMD_RM, SHM_CMD_AT, SHM_CMD_DT, SHM_CMD_GET, SHM_CMD_INFO,
    SHM_CMD_RM,
};
