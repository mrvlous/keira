// SPDX-License-Identifier: GPL-2.0-only

//! Dynamic `/proc/[pid]/maps` process virtual memory address space inspection node.

use super::hooks::TASK_MAPS_HOOK;

/// Reads task virtual memory mappings for the given process ID into the buffer.
pub fn read_task_maps(pid: usize, buf: &mut [u8]) -> Result<usize, &'static str> {
    if let Some(hook) = unsafe { TASK_MAPS_HOOK } {
        if let Some(bytes) = hook(pid, buf) {
            return Ok(bytes);
        }
    }
    Err("Process not found")
}
