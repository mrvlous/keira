// SPDX-License-Identifier: GPL-2.0-only
/*
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra <mrvlous@proton.me>
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 */

//! Subsystem module: flags.

pub mod types {
/// Maximum number of tokens parsed by `CliArgs`.
pub const MAX_ARG_TOKENS: usize = 16;

/// Zero-allocation helper for parsing command-line flags and options.
pub struct CliArgs<'a> {
    pub(crate) tokens: [&'a str; MAX_ARG_TOKENS],
    pub(crate) token_count: usize,
}

}

pub use self::types::*;
