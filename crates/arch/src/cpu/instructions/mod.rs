// SPDX-License-Identifier: GPL-2.0-only

//! Low-level x86/x86_64 CPU assembly intrinsics subsystem.

pub mod intrinsics;

pub use intrinsics::{cli, hlt, invlpg, pause, rdtsc, sti};
