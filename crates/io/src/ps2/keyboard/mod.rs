// SPDX-License-Identifier: GPL-2.0-only

//! PS/2 Keyboard Controller module.

mod driver;
mod scancode;
#[cfg(test)]
mod tests;

pub use self::driver::*;
pub use self::scancode::*;
