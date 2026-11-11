// SPDX-License-Identifier: GPL-2.0-only

//! Signal action handler registration and process disposition tracking.

pub mod disposition;

pub use disposition::{
    get_signal_handler, reset_signal_handlers, sys_sigaction, MAX_SIGNAL_TASKS, SIGNAL_HANDLERS,
};
