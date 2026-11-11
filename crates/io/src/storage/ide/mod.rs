// SPDX-License-Identifier: GPL-2.0-only

//! Primary Master ATA IDE Hard Disk Drive controller driver using LBA28.

mod device;
mod driver;
#[cfg(test)]
mod tests;

pub use self::device::*;
pub use self::driver::*;
