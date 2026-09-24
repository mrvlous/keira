// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Zero-allocation command-line argument container and token structures.

/// Maximum number of tokens parsed by `CliArgs`.
pub const MAX_ARG_TOKENS: usize = 16;

/// Zero-allocation helper for parsing command-line flags and options.
pub struct CliArgs<'a> {
    pub(crate) tokens: [&'a str; MAX_ARG_TOKENS],
    pub(crate) token_count: usize,
}
