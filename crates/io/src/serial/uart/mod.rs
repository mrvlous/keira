// SPDX-License-Identifier: GPL-2.0-only

//! 16550A Serial UART driver and formatting module.

mod driver;
#[cfg(test)]
mod tests;
mod writer;

pub use self::driver::*;
pub use self::writer::*;
