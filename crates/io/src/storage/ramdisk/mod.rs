// SPDX-License-Identifier: GPL-2.0-only

//! In-memory RAM disk block device emulator and FAT16 initialization formatter.

mod device;
mod format;
#[cfg(test)]
mod tests;

pub use self::device::*;
pub use self::format::*;
