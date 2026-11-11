// SPDX-License-Identifier: GPL-2.0-only

//! Keyed-Hash Message Authentication Code (HMAC) subsystem.

mod engine;

pub use self::engine::*;

#[cfg(test)]
mod tests;
