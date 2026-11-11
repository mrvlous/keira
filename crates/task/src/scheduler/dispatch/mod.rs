// SPDX-License-Identifier: GPL-2.0-only

//! Preemptive timer interrupt dispatch and task context scheduling.

pub mod tick;

pub use tick::schedule_tick;
