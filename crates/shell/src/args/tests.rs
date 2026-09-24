// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Unit tests for CLI argument parsing, flag detection, and option extraction.

#[cfg(test)]
pub mod test {
    use crate::args::*;

    #[test]
    fn test_cli_args_parsing() {
        let input = "-l -a --long target_file.txt";
        let mut parts = input.split_whitespace();
        let cli = CliArgs::parse(&mut parts);
        assert!(cli.has_flag('l', "long"));
        assert!(cli.has_flag('a', "all"));
        assert_eq!(cli.first_positional(), Some("target_file.txt"));
        assert_eq!(cli.len(), 4);
        assert!(!cli.is_empty());
    }

    #[test]
    fn test_cli_args_options() {
        let input = "-n 10 --format=json data.txt";
        let mut parts = input.split_whitespace();
        let cli = CliArgs::parse(&mut parts);
        assert_eq!(cli.get_opt('n', "number"), Some("10"));
        assert_eq!(cli.get_opt('f', "format"), Some("json"));
        assert_eq!(cli.first_positional(), Some("data.txt"));
    }

    #[test]
    fn test_cli_args_positionals() {
        let input = "src_file.txt dst_file.txt extra.txt";
        let mut parts = input.split_whitespace();
        let cli = CliArgs::parse(&mut parts);
        assert_eq!(cli.first_positional(), Some("src_file.txt"));
        assert_eq!(cli.second_positional(), Some("dst_file.txt"));
        assert_eq!(cli.positional(2), Some("extra.txt"));
        assert_eq!(cli.positional(3), None);
    }
}
