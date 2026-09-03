// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//!
//! Lightweight `#![no_std]` CLI argument and flag parser for Keira Shell commands.

/// Zero-allocation helper for parsing command-line flags and options.
pub struct CliArgs<'a> {
    tokens: [&'a str; 16],
    token_count: usize,
}

impl<'a> CliArgs<'a> {
    /// Parse arguments from a whitespace-split iterator.
    pub fn parse(parts: &mut core::str::SplitWhitespace<'a>) -> Self {
        let mut tokens = [""; 16];
        let mut token_count = 0;

        for part in parts {
            if token_count < 16 {
                tokens[token_count] = part;
                token_count += 1;
            }
        }

        Self {
            tokens,
            token_count,
        }
    }

    /// Check if a short flag (e.g. `c` for `-c`) or long flag (e.g. `clear` for `--clear`) is present.
    pub fn has_flag(&self, short: char, long: &str) -> bool {
        for i in 0..self.token_count {
            let tok = self.tokens[i];
            if tok.starts_with("--") {
                if &tok[2..] == long {
                    return true;
                }
            } else if tok.starts_with('-') && tok.len() > 1 {
                if tok[1..].chars().any(|ch| ch == short) {
                    return true;
                }
            }
        }
        false
    }

    /// Retrieve an option value by short key or long key (e.g. `-n 5` or `--count 5` or `-n=5` or `--count=5`).
    pub fn get_opt(&self, short: char, long: &str) -> Option<&'a str> {
        for i in 0..self.token_count {
            let tok = self.tokens[i];

            // Form: --long=value
            if tok.starts_with("--") {
                let rest = &tok[2..];
                if let Some(eq_idx) = rest.find('=') {
                    if &rest[..eq_idx] == long {
                        return Some(&rest[eq_idx + 1..]);
                    }
                } else if rest == long && i + 1 < self.token_count {
                    let next_tok = self.tokens[i + 1];
                    if !next_tok.starts_with('-') {
                        return Some(next_tok);
                    }
                }
            }
            // Form: -s=value or -s value
            else if tok.starts_with('-') && tok.len() > 1 {
                let rest = &tok[1..];
                if let Some(eq_idx) = rest.find('=') {
                    let prefix = &rest[..eq_idx];
                    if prefix.chars().any(|ch| ch == short) {
                        return Some(&rest[eq_idx + 1..]);
                    }
                } else if rest.len() == 1 && rest.chars().next() == Some(short) {
                    if i + 1 < self.token_count {
                        let next_tok = self.tokens[i + 1];
                        if !next_tok.starts_with('-') {
                            return Some(next_tok);
                        }
                    }
                }
            }
        }
        None
    }

    /// Retrieve the first positional argument that is not a flag/option.
    pub fn first_positional(&self) -> Option<&'a str> {
        let mut skip_next = false;
        for i in 0..self.token_count {
            let tok = self.tokens[i];
            if skip_next {
                skip_next = false;
                continue;
            }

            if tok.starts_with('-') {
                // If it's a flag that takes a value in next token (e.g. -n 5)
                if !tok.contains('=') && tok.len() == 2 && i + 1 < self.token_count {
                    let next = self.tokens[i + 1];
                    if !next.starts_with('-')
                        && (tok == "-n" || tok == "-p" || tok == "-u" || tok == "-s" || tok == "-d")
                    {
                        skip_next = true;
                    }
                }
                continue;
            }

            return Some(tok);
        }
        None
    }

    /// Retrieve the N-th positional argument that is not a flag/option.
    pub fn positional(&self, idx: usize) -> Option<&'a str> {
        let mut cur = 0;
        let mut skip_next = false;
        for i in 0..self.token_count {
            let tok = self.tokens[i];
            if skip_next {
                skip_next = false;
                continue;
            }
            if tok.starts_with('-') {
                if !tok.contains('=') && tok.len() == 2 && i + 1 < self.token_count {
                    let next = self.tokens[i + 1];
                    if !next.starts_with('-')
                        && (tok == "-n" || tok == "-p" || tok == "-u" || tok == "-s" || tok == "-d")
                    {
                        skip_next = true;
                    }
                }
                continue;
            }
            if cur == idx {
                return Some(tok);
            }
            cur += 1;
        }
        None
    }

    /// Retrieve the second positional argument that is not a flag/option.
    pub fn second_positional(&self) -> Option<&'a str> {
        self.positional(1)
    }

    /// Get total parsed token count.
    pub fn len(&self) -> usize {
        self.token_count
    }

    /// Check if no arguments were provided.
    pub fn is_empty(&self) -> bool {
        self.token_count == 0
    }
}
