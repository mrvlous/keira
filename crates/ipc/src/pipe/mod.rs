// SPDX-License-Identifier: GPL-2.0-only

//! Pipe streaming and zero-copy splice IPC.

pub mod fifo;
pub mod zero_copy;

pub use fifo::*;
pub use zero_copy::*;

pub mod splice {
    pub use super::zero_copy::*;
}

#[cfg(test)]
mod tests;
