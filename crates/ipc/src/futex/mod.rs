// SPDX-License-Identifier: GPL-2.0-only

//! Fast user-space synchronization (`futex`).

pub mod hash;
pub mod wait;
pub mod wake;

pub use hash::*;
pub use wait::*;
pub use wake::*;

pub mod sync {
    pub use super::hash::*;
    pub use super::wait::*;
    pub use super::wake::*;
}

#[cfg(test)]
mod tests;
