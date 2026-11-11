// SPDX-License-Identifier: GPL-2.0-only

//! POSIX Shared Memory segments and counting semaphores.

pub mod manager;
pub mod segment;

pub use segment::*;

pub mod segment_compat {
    pub use super::segment::*;
}

#[cfg(test)]
mod tests;
