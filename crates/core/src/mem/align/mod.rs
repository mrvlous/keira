// SPDX-License-Identifier: GPL-2.0-only

//! Memory alignment arithmetic utilities and boundary validation subsystem.

pub mod arithmetic;

pub use arithmetic::{align_down, align_up, is_aligned};
