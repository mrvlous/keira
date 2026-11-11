// SPDX-License-Identifier: GPL-2.0-only

//! Primitives for copying data between kernel address space and user address space.

pub mod buffer;
pub mod typed;

pub use buffer::{copy_from_user, copy_to_user};
pub use typed::{copy_val_from_user, copy_val_to_user};
