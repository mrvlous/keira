// SPDX-License-Identifier: GPL-2.0-only

//! Dynamic `/proc/[pid]/status` process state inspection node.

use super::hooks::TASK_STATUS_HOOK;

/// Reads task status information for the given process ID into the buffer.
pub fn read_task_status(pid: usize, buf: &mut [u8]) -> Result<usize, &'static str> {
    if let Some(hook) = unsafe { TASK_STATUS_HOOK } {
        if let Some(bytes) = hook(pid, buf) {
            return Ok(bytes);
        }
    }
    Err("Process not found")
}
