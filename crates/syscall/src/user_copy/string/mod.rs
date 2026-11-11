// SPDX-License-Identifier: GPL-2.0-only

//! Primitives for safely reading null-terminated user space strings.

pub mod reader;

pub use reader::{read_user_string, read_user_string_bounded};
