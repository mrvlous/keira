// SPDX-License-Identifier: GPL-2.0-only

//! Generic block device trait abstraction and system block device registry.

mod registry;
#[cfg(test)]
mod tests;
mod traits;

pub use self::registry::*;
pub use self::traits::*;
