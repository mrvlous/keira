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
    use crate::scheduler::core::{CURRENT_TASK_IDX, MAX_TASKS, TASKS};

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

    pub unsafe fn get_current_pgid() -> usize {
        if let Some(ref task) = TASKS[CURRENT_TASK_IDX] {
            task.pgid
        } else {
            0
        }
    }

    pub unsafe fn get_task_pgid(pid: usize) -> Result<usize, &'static str> {
        let target_pid = if pid == 0 { CURRENT_TASK_IDX } else { pid };
        if target_pid >= MAX_TASKS {
            return Err("Invalid process ID");
        }
        if let Some(ref task) = TASKS[target_pid] {
            Ok(task.pgid)
        } else {
            Err("No such process")
        }
    }

    pub unsafe fn set_task_pgid(pid: usize, pgid: usize) -> Result<(), &'static str> {
        let current_pid = CURRENT_TASK_IDX;
        let target_pid = if pid == 0 { current_pid } else { pid };
        if target_pid >= MAX_TASKS {
            return Err("Invalid process ID");
        }

        let current_sid = if let Some(ref cur) = TASKS[current_pid] {
            cur.sid
        } else {
            return Err("No active task");
        };

        let target_pgid = if pgid == 0 { target_pid } else { pgid };
        if target_pgid >= MAX_TASKS {
            return Err("Invalid process group ID");
        }

        if let Some(ref mut task) = TASKS[target_pid] {
            // Cannot change PGID of a session leader
            if task.id == task.sid && task.id != target_pgid {
                return Err("Process is session leader");
            }
            // Target must belong to the same session as current process
            if task.sid != current_sid {
                return Err("Process belongs to different session");
            }
            // Target must be current process or a child of current process
            if task.id != current_pid && task.parent_id != current_pid {
                return Err("Process is not child of caller");
            }
            task.pgid = target_pgid;
            Ok(())
        } else {
            Err("No such process")
        }
    }

    pub unsafe fn get_current_sid() -> usize {
        if let Some(ref task) = TASKS[CURRENT_TASK_IDX] {
            task.sid
        } else {
            0
        }
    }

    pub unsafe fn get_task_sid(pid: usize) -> Result<usize, &'static str> {
        let target_pid = if pid == 0 { CURRENT_TASK_IDX } else { pid };
        if target_pid >= MAX_TASKS {
            return Err("Invalid process ID");
        }
        if let Some(ref task) = TASKS[target_pid] {
            Ok(task.sid)
        } else {
            Err("No such process")
        }
    }

    pub unsafe fn setsid_current() -> Result<usize, &'static str> {
        if let Some(ref mut task) = TASKS[CURRENT_TASK_IDX] {
            if task.id == task.pgid {
                return Err("Process is already group leader");
            }
            task.sid = task.id;
            task.pgid = task.id;
            Ok(task.sid)
        } else {
            Err("No active task")
        }
    }
}

pub use self::creds::*;
