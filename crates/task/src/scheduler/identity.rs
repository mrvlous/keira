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

//! Process identity credentials (UID, GID, EUID, EGID).

pub mod creds {
    use crate::scheduler::core::{CURRENT_TASK_IDX, TASKS};

    pub unsafe fn get_current_uid() -> u32 {
        if let Some(ref task) = TASKS[CURRENT_TASK_IDX] {
            task.uid
        } else {
            0
        }
    }

    pub unsafe fn get_current_euid() -> u32 {
        if let Some(ref task) = TASKS[CURRENT_TASK_IDX] {
            task.euid
        } else {
            0
        }
    }

    pub unsafe fn set_current_uid(new_uid: u32) -> Result<(), &'static str> {
        if let Some(ref mut task) = TASKS[CURRENT_TASK_IDX] {
            if task.euid == 0 {
                task.uid = new_uid;
                task.euid = new_uid;
                Ok(())
            } else if new_uid == task.uid {
                task.euid = new_uid;
                Ok(())
            } else {
                Err("Operation not permitted")
            }
        } else {
            Err("No active task")
        }
    }

    pub unsafe fn get_current_gid() -> u32 {
        if let Some(ref task) = TASKS[CURRENT_TASK_IDX] {
            task.gid
        } else {
            0
        }
    }

    pub unsafe fn get_current_egid() -> u32 {
        if let Some(ref task) = TASKS[CURRENT_TASK_IDX] {
            task.egid
        } else {
            0
        }
    }

    pub unsafe fn set_current_gid(new_gid: u32) -> Result<(), &'static str> {
        if let Some(ref mut task) = TASKS[CURRENT_TASK_IDX] {
            if task.euid == 0 {
                task.gid = new_gid;
                task.egid = new_gid;
                Ok(())
            } else if new_gid == task.gid {
                task.egid = new_gid;
                Ok(())
            } else {
                Err("Operation not permitted")
            }
        } else {
            Err("No active task")
        }
    }
}

pub use self::creds::*;
