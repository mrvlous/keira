// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Process user and group credentials management (UID/GID/EUID/EGID).

use crate::scheduler::core::{CURRENT_TASK_IDX, TASKS};

/// Query the real UID of the current task.
///
/// # Safety
/// Caller must ensure synchronized access to `TASKS`.
pub unsafe fn get_current_uid() -> u32 {
    if let Some(ref task) = TASKS[CURRENT_TASK_IDX] {
        task.uid
    } else {
        0
    }
}

/// Query the effective UID of the current task.
///
/// # Safety
/// Caller must ensure synchronized access to `TASKS`.
pub unsafe fn get_current_euid() -> u32 {
    if let Some(ref task) = TASKS[CURRENT_TASK_IDX] {
        task.euid
    } else {
        0
    }
}

/// Set the real and effective UID of the current task according to POSIX privilege rules.
///
/// # Safety
/// Caller must ensure synchronized access to `TASKS`.
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

/// Query the real GID of the current task.
///
/// # Safety
/// Caller must ensure synchronized access to `TASKS`.
pub unsafe fn get_current_gid() -> u32 {
    if let Some(ref task) = TASKS[CURRENT_TASK_IDX] {
        task.gid
    } else {
        0
    }
}

/// Query the effective GID of the current task.
///
/// # Safety
/// Caller must ensure synchronized access to `TASKS`.
pub unsafe fn get_current_egid() -> u32 {
    if let Some(ref task) = TASKS[CURRENT_TASK_IDX] {
        task.egid
    } else {
        0
    }
}

/// Set the real and effective GID of the current task according to POSIX privilege rules.
///
/// # Safety
/// Caller must ensure synchronized access to `TASKS`.
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
