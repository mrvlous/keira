// SPDX-License-Identifier: GPL-2.0-only

//! USB Host Controller (UHCI, OHCI, EHCI, xHCI) detection and bus enumeration.

mod discovery;
#[cfg(test)]
mod tests;

pub use self::discovery::*;
