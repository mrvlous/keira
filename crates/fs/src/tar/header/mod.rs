// SPDX-License-Identifier: GPL-2.0-only

//! POSIX USTAR header structure and parsing utilities.

pub mod octal;
pub mod types;

pub use self::octal::octal_str_to_u64;
pub use self::types::{UstarHeader, TYPEFLAG_DIRECTORY, TYPEFLAG_REGULAR, TYPEFLAG_REGULAR_ALT};
