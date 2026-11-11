// SPDX-License-Identifier: GPL-2.0-only

//! Shell state management, input ring buffers, line editing context and history state.

pub mod env;
pub mod session;

#[cfg(test)]
mod tests;

pub use env::*;
pub use session::*;
