// SPDX-License-Identifier: GPL-2.0-only

//! Trusted Platform Module (TPM 2.0) interface and subsystem.

pub mod hardware;
pub mod pcr;
pub mod sealed;

pub use self::hardware::*;
pub use self::pcr::*;
pub use self::sealed::*;
