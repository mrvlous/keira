// SPDX-License-Identifier: GPL-2.0-only

//! NVM Express (NVMe) PCIe solid-state drive controller driver.

mod controller;
#[cfg(test)]
mod tests;

pub use self::controller::*;
