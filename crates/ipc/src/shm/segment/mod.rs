// SPDX-License-Identifier: GPL-2.0-only
/*
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra <mrvlous@proton.me>
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 */

//! Shared memory segment structures and semaphore descriptors.

pub mod types;

pub use super::manager::*;
pub use types::{
    Semaphore, ShmSegment, SEM_CMD_RM, SHM_CMD_AT, SHM_CMD_DT, SHM_CMD_GET, SHM_CMD_INFO,
    SHM_CMD_RM,
};
