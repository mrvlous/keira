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

//! Event condition waiting and sleep loop macros.

/// Sleep until condition evaluates to true.
#[macro_export]
macro_rules! wait_event {
    ($wq:expr, $condition:expr) => {{
        let curr_pid = $crate::scheduler::core::CURRENT_TASK_IDX;
        let entry = $crate::wait::WaitQueueEntry::new(curr_pid, 0);
        loop {
            if $condition {
                break;
            }
            $wq.prepare_to_wait(entry, $crate::types::TaskState::Blocked);
            if $condition {
                $wq.finish_wait(curr_pid);
                break;
            }
            #[cfg(target_os = "none")]
            unsafe {
                core::arch::asm!("sti; int 32; cli");
            }
            $wq.finish_wait(curr_pid);
        }
    }};
}

/// Sleep until condition evaluates to true or timeout expires in milliseconds.
#[macro_export]
macro_rules! wait_event_timeout {
    ($wq:expr, $condition:expr, $timeout_ms:expr) => {{
        let curr_pid = $crate::scheduler::core::CURRENT_TASK_IDX;
        let entry = $crate::wait::WaitQueueEntry::new(curr_pid, 0);
        let start_ms = keira_arch::timers::pit::uptime_ms();
        let mut timed_out = false;
        loop {
            if $condition {
                break;
            }
            if keira_arch::timers::pit::uptime_ms().saturating_sub(start_ms) >= $timeout_ms {
                timed_out = true;
                break;
            }
            $wq.prepare_to_wait(entry, $crate::types::TaskState::Blocked);
            if $condition {
                $wq.finish_wait(curr_pid);
                break;
            }
            #[cfg(target_os = "none")]
            unsafe {
                core::arch::asm!("sti; int 32; cli");
            }
            $wq.finish_wait(curr_pid);
        }
        !timed_out
    }};
}
