// SPDX-License-Identifier: GPL-2.0-only

//! Dynamic `/proc/[pid]/statm` virtual memory page metrics node.

use super::hooks::TASK_STATM_HOOK;

/// Reads task memory page metrics for the given process ID into the buffer.
pub fn read_task_statm(pid: usize, buf: &mut [u8]) -> Result<usize, &'static str> {
    if let Some(hook) = unsafe { TASK_STATM_HOOK } {
        if let Some(bytes) = hook(pid, buf) {
            return Ok(bytes);
        }
    }
    Err("Process not found")
}
