// SPDX-License-Identifier: GPL-2.0-only

//! System call routing and dispatch dispatching.

pub mod entry;

pub use entry::{sanitize_ret, syscall_dispatcher};
