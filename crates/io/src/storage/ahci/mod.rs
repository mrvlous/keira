// SPDX-License-Identifier: GPL-2.0-only

//! Advanced Host Controller Interface (AHCI) Serial ATA (SATA) disk driver.

mod device;
mod driver;
mod registers;
#[cfg(test)]
mod tests;

pub use self::device::*;
pub use self::driver::*;
pub use self::registers::*;
