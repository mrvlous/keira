// SPDX-License-Identifier: GPL-2.0-only

//! Dynamic `/proc/[pid]/cmdline` process command-line arguments node.

use super::hooks::TASK_CMDLINE_HOOK;

/// Reads task command line string for the given process ID into the buffer.
pub fn read_task_cmdline(pid: usize, buf: &mut [u8]) -> Result<usize, &'static str> {
    if let Some(hook) = unsafe { TASK_CMDLINE_HOOK } {
        if let Some(bytes) = hook(pid, buf) {
            return Ok(bytes);
        }
    }
    Err("Process not found")
}
