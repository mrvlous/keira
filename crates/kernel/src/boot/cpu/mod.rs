// SPDX-License-Identifier: GPL-2.0-only

//! CPUID detection and architecture confirmation subsystem.

pub mod detect;

pub use detect::detect_cpu;
