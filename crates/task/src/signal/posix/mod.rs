// SPDX-License-Identifier: GPL-2.0-only

//! Standard POSIX signal definitions, numbers and mask commands.

pub mod mask;
pub mod numbers;

pub use mask::*;
pub use numbers::*;
