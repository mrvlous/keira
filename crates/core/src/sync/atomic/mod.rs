// SPDX-License-Identifier: GPL-2.0-only

//! Low-level atomic memory barriers, fences and processor hints subsystem.

pub mod barrier;

pub use barrier::{memory_barrier_compiler, memory_barrier_hardware, spin_loop_hint};
