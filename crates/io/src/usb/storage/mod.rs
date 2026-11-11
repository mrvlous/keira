// SPDX-License-Identifier: GPL-2.0-only

//! USB Mass Storage Bulk-Only Transport (BOT) and SCSI command framing.

mod bot;
#[cfg(test)]
mod tests;

pub use self::bot::*;
