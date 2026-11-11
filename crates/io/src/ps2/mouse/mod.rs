// SPDX-License-Identifier: GPL-2.0-only

//! PS/2 Auxiliary Mouse Controller module.

mod driver;
mod packet;
#[cfg(test)]
mod tests;

pub use self::driver::*;
pub use self::packet::*;
