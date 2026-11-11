// SPDX-License-Identifier: GPL-2.0-only

//! Zero-copy page frame swapping and buffer slicing primitives.

pub mod splice;

pub use splice::{sys_splice, sys_vmsplice};
