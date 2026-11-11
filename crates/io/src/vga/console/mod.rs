// SPDX-License-Identifier: GPL-2.0-only

//! Unified character console output, video RAM manipulation and boot milestone logging.

mod buffer;
mod pipe;
mod print;
#[cfg(test)]
mod tests;

pub use self::buffer::*;
pub use self::pipe::*;
pub use self::print::*;
