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

//! Task-level signal state and delivery mechanisms.

pub mod context;
pub mod delivery;

pub use context::{set_saved_sigcontext, take_saved_sigcontext};
pub use delivery::{
    get_current_pending_signals, get_current_signal_mask, send_signal, sys_sigprocmask, SIG_BLOCK,
    SIG_SETMASK, SIG_UNBLOCK,
};
