// SPDX-License-Identifier: GPL-2.0-only

//! Ring 3 user mode execution transition.

extern "C" {
    fn jump_to_user(entry: u64, stack: u64);
}

/// Jump to user-mode code entry point with prepared user stack.
///
/// # Safety
/// The caller must ensure that the entry point and user stack reside in mapped,
/// user-accessible virtual memory before transitioning to Ring 3.
pub unsafe fn execute_user_mode(entry_point: u64, user_stack_top: u64) {
    jump_to_user(entry_point, user_stack_top);
}
