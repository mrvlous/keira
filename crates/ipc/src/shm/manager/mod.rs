// SPDX-License-Identifier: GPL-2.0-only

//! Shared memory segment and semaphore table management and syscalls.

pub mod ops;
pub mod table;

pub use ops::sys_shm_sem;
pub use table::{
    create_sem, create_shm, get_sem_table, get_shm_frame, get_shm_table, remove_sem, remove_shm,
    SEM_TABLE, SHM_TABLE,
};
