// SPDX-License-Identifier: GPL-2.0-only

//! Ring 3 user privilege and execution context management.

pub mod context;

pub use context::enter_userspace;
