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

//! Scheduler signal delivery and sigcontext frame setup.

pub mod context {
    use crate::scheduler::core::{CURRENT_TASK_IDX, TASKS};
    use crate::types::InterruptContext;

    pub unsafe fn set_saved_sigcontext(ctx: InterruptContext) {
        if let Some(ref mut task) = TASKS[CURRENT_TASK_IDX] {
            task.saved_sigcontext = Some(ctx);
        }
    }

    pub unsafe fn take_saved_sigcontext() -> Option<InterruptContext> {
        if let Some(ref mut task) = TASKS[CURRENT_TASK_IDX] {
            task.saved_sigcontext.take()
        } else {
            None
        }
    }
}

pub mod delivery {
    use crate::scheduler::core::{CURRENT_TASK_IDX, MAX_TASKS, TASKS};
    use crate::signal::action::get_signal_handler;
    pub use crate::signal::posix::{SIG_BLOCK, SIG_SETMASK, SIG_UNBLOCK};
    use crate::types::{InterruptContext, TaskState};

    pub unsafe fn send_signal(pid: usize, sig: u32) -> Result<(), &'static str> {
        if pid >= MAX_TASKS {
            return Err("Target PID out of scheduler table range");
        }
        if let Some(ref mut task) = TASKS[pid] {
            let is_unblockable = sig == 9 || sig == 19;
            if !is_unblockable && (task.signal_mask & (1 << sig)) != 0 {
                task.pending_signals |= 1 << sig;
                return Ok(());
            }

            let handler = get_signal_handler(pid, sig);
            if handler != 0 {
                if task.saved_sigcontext.is_none() {
                    #[cfg(target_arch = "x86_64")]
                    let saved_ctx = InterruptContext {
                        rip: task.rsp,
                        ..Default::default()
                    };
                    #[cfg(target_arch = "x86")]
                    let saved_ctx = InterruptContext {
                        rip: task.rsp as u32,
                        ..Default::default()
                    };
                    task.saved_sigcontext = Some(saved_ctx);
                }
                return Ok(());
            }

            if pid == 0 {
                return Err("Signal delivery to bootstrap kernel shell is restricted");
            }

            match sig {
                1..=15 => {
                    task.state = TaskState::Zombie(-(sig as i32));
                    Ok(())
                }
                18 => {
                    if task.state == TaskState::Blocked {
                        task.state = TaskState::Ready;
                    }
                    Ok(())
                }
                19 => {
                    task.state = TaskState::Blocked;
                    Ok(())
                }
                _ => Ok(()),
            }
        } else {
            Err("Process with specified PID does not exist")
        }
    }

    pub unsafe fn get_current_signal_mask() -> u32 {
        if let Some(ref t) = TASKS[CURRENT_TASK_IDX] {
            t.signal_mask
        } else {
            0
        }
    }

    pub unsafe fn get_current_pending_signals() -> u32 {
        if let Some(ref t) = TASKS[CURRENT_TASK_IDX] {
            t.pending_signals
        } else {
            0
        }
    }

    pub unsafe fn sys_sigprocmask(
        how: i32,
        set: u32,
        old_set: *mut u32,
    ) -> Result<u32, &'static str> {
        if let Some(ref mut task) = TASKS[CURRENT_TASK_IDX] {
            if !old_set.is_null() {
                *old_set = task.signal_mask;
            }

            let unblockable = (1 << 9) | (1 << 19);
            let clean_set = set & !unblockable;

            match how {
                SIG_BLOCK => {
                    task.signal_mask |= clean_set;
                }
                SIG_UNBLOCK => {
                    task.signal_mask &= !clean_set;
                }
                SIG_SETMASK => {
                    task.signal_mask = clean_set;
                }
                _ => return Err("Invalid how parameter for sigprocmask"),
            }

            let unmasked_pending = task.pending_signals & !task.signal_mask;
            if unmasked_pending != 0 {
                for s in 1..32 {
                    if (unmasked_pending & (1 << s)) != 0 {
                        task.pending_signals &= !(1 << s);
                        let _ = send_signal(CURRENT_TASK_IDX, s);
                        break;
                    }
                }
            }
            Ok(0)
        } else {
            Err("Current task invalid")
        }
    }
}

pub use self::context::*;
pub use self::delivery::*;
