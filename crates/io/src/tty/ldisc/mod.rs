// SPDX-License-Identifier: GPL-2.0-only

//! Terminal line discipline subsystem.

mod discipline;
mod termios;
#[cfg(test)]
mod tests;

pub use self::discipline::*;
pub use self::termios::*;
