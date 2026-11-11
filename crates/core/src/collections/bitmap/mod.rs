// SPDX-License-Identifier: GPL-2.0-only

//! Fixed-size static bitmap allocation subsystem.
//!
//! Provides zero-allocation tracking for hardware frame tables and resource bitmasks.

pub mod bitset;

pub use bitset::Bitmap;
