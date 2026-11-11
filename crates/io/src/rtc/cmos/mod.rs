// SPDX-License-Identifier: GPL-2.0-only

//! CMOS Real-Time Clock hardware driver module.

mod driver;
#[cfg(test)]
mod tests;
mod time;

pub use self::driver::*;
pub use self::time::*;
