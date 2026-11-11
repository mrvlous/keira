// SPDX-License-Identifier: GPL-2.0-only

//! Shared memory segment structures and semaphore descriptors.

pub mod types;

pub use super::manager::*;
pub use types::{
    Semaphore, ShmSegment, SEM_CMD_RM, SHM_CMD_AT, SHM_CMD_DT, SHM_CMD_GET, SHM_CMD_INFO,
    SHM_CMD_RM,
};
