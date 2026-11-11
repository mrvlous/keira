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

//! Shared memory segment and semaphore table management and syscalls.

pub mod ops;
pub mod table;

pub use ops::sys_shm_sem;
pub use table::{
    create_sem, create_shm, get_sem_table, get_shm_frame, get_shm_table, remove_sem, remove_shm,
    SEM_TABLE, SHM_TABLE,
};
