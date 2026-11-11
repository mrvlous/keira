// SPDX-License-Identifier: GPL-2.0-only

//! Architecture-specific initial user stack formatting.

pub mod amd64;
pub mod i386;

pub use amd64::setup_user_stack_64;
pub use i386::setup_user_stack_32;
