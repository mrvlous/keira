// SPDX-License-Identifier: GPL-2.0-only

//! Virtual terminal session multiplexing module.

mod session;
#[cfg(test)]
mod tests;

pub use self::session::*;
