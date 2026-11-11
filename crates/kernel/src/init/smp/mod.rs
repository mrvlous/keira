// SPDX-License-Identifier: GPL-2.0-only

//! Symmetric Multi-Processing (SMP) and hardware timers initialization stage.

pub mod topology;

pub use topology::init_smp_and_timers;
