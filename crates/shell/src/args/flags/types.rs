// SPDX-License-Identifier: GPL-2.0-only

//! Zero-allocation command-line argument container and token structures.

/// Maximum number of tokens parsed by `CliArgs`.
pub const MAX_ARG_TOKENS: usize = 16;

/// Zero-allocation helper for parsing command-line flags and options.
pub struct CliArgs<'a> {
    pub(crate) tokens: [&'a str; MAX_ARG_TOKENS],
    pub(crate) token_count: usize,
}
