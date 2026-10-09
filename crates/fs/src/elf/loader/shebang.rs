// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Shebang (`#!`) interpreter header parser for executable scripts.

/// Parse a shebang line (`#!<interpreter> [arg]`) from the initial bytes of a script file.
///
/// Returns a tuple of `(interpreter_path, optional_argument)` if the slice begins with `#!`.
pub fn parse_shebang(bytes: &[u8]) -> Option<(&str, Option<&str>)> {
    if bytes.len() < 2 || bytes[0] != b'#' || bytes[1] != b'!' {
        return None;
    }

    // Find end of the first line (newline or carriage return or end of buffer)
    let mut eol = 2;
    while eol < bytes.len() && bytes[eol] != b'\n' && bytes[eol] != b'\r' {
        eol += 1;
    }

    let line = &bytes[2..eol];

    // Skip leading ASCII whitespace
    let mut start = 0;
    while start < line.len() && (line[start] == b' ' || line[start] == b'\t') {
        start += 1;
    }

    if start >= line.len() {
        return None;
    }

    // Extract interpreter path until whitespace or end of line
    let mut interp_end = start;
    while interp_end < line.len() && line[interp_end] != b' ' && line[interp_end] != b'\t' {
        interp_end += 1;
    }

    let interp_str = core::str::from_utf8(&line[start..interp_end]).ok()?;
    if interp_str.is_empty() {
        return None;
    }

    // Skip whitespace between interpreter and optional argument
    let mut arg_start = interp_end;
    while arg_start < line.len() && (line[arg_start] == b' ' || line[arg_start] == b'\t') {
        arg_start += 1;
    }

    let opt_arg = if arg_start < line.len() {
        // Read argument until next whitespace or end of line
        let mut arg_end = arg_start;
        while arg_end < line.len() && line[arg_end] != b' ' && line[arg_end] != b'\t' {
            arg_end += 1;
        }
        core::str::from_utf8(&line[arg_start..arg_end]).ok()
    } else {
        None
    };

    Some((interp_str, opt_arg))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_shebang_simple() {
        let data = b"#!/bin/sh\necho hello\n";
        let res = parse_shebang(data);
        assert_eq!(res, Some(("/bin/sh", None)));
    }

    #[test]
    fn test_parse_shebang_with_spaces() {
        let data = b"#!  /bin/sh.elf\r\n";
        let res = parse_shebang(data);
        assert_eq!(res, Some(("/bin/sh.elf", None)));
    }

    #[test]
    fn test_parse_shebang_with_argument() {
        let data = b"#!/bin/sh -e\necho test\n";
        let res = parse_shebang(data);
        assert_eq!(res, Some(("/bin/sh", Some("-e"))));
    }

    #[test]
    fn test_parse_shebang_invalid() {
        assert_eq!(parse_shebang(b"\x7fELF\x02\x01"), None);
        assert_eq!(parse_shebang(b"#"), None);
        assert_eq!(parse_shebang(b""), None);
        assert_eq!(parse_shebang(b"#!   \n"), None);
    }
}
