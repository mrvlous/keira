// SPDX-License-Identifier: GPL-2.0-only

//! Task descriptor representations and state machine.

pub mod descriptor;
pub mod state;

pub use descriptor::{Task, MAX_TASKS};
pub use state::TaskState;
