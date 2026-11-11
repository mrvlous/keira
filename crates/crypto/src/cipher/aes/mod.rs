// SPDX-License-Identifier: GPL-2.0-only

//! AES-128 (Rijndael) symmetric block cipher subsystem.

mod core;

pub use self::core::*;

#[cfg(test)]
mod tests;
