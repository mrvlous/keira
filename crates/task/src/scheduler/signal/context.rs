// SPDX-License-Identifier: GPL-2.0-only

//! Signal frame register context save and restore.

use crate::scheduler::core::{CURRENT_TASK_IDX, TASKS};
use crate::types::InterruptContext;

/// Store a saved interrupt context for signal return in the current task.
///
/// # Safety
/// Caller must ensure single-threaded execution or synchronized scheduler access.
pub unsafe fn set_saved_sigcontext(ctx: InterruptContext) {
    if let Some(ref mut task) = TASKS[CURRENT_TASK_IDX] {
        task.saved_sigcontext = Some(ctx);
    }
}

/// Retrieve and clear the saved interrupt context for signal return in the current task.
///
/// # Safety
/// Caller must ensure single-threaded execution or synchronized scheduler access.
pub unsafe fn take_saved_sigcontext() -> Option<InterruptContext> {
    if let Some(ref mut task) = TASKS[CURRENT_TASK_IDX] {
        task.saved_sigcontext.take()
    } else {
        None
    }
}
