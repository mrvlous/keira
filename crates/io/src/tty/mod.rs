// SPDX-License-Identifier: GPL-2.0-only

//! Virtual terminal (TTY) manager and session multiplexing.

pub mod ldisc;
pub mod term;

pub use self::ldisc::*;
pub use self::term::*;
