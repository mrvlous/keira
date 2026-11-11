// SPDX-License-Identifier: GPL-2.0-only

//! Seccomp mode and state descriptors.

pub mod modes;

pub use modes::{SeccompMode, SeccompState, SECCOMP_SET_MODE_FILTER, SECCOMP_SET_MODE_STRICT};
