// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Uniform Resource Locator (URL) parser for HTTP and HTTPS network requests.

/// Parsed URL components including protocol, host, port, and path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParsedUrl<'a> {
    pub is_https: bool,
    pub host: &'a str,
    pub port: u16,
    pub path: &'a str,
}

impl<'a> ParsedUrl<'a> {
    /// Parse an HTTP or HTTPS URL string into its individual components.
    pub fn parse(raw_url: &'a str) -> Result<Self, &'static str> {
        let trimmed = raw_url.trim();
        if trimmed.is_empty() {
            return Err("Empty URL");
        }

        let (is_https, without_scheme) = if let Some(stripped) = trimmed.strip_prefix("https://") {
            (true, stripped)
        } else if let Some(stripped) = trimmed.strip_prefix("http://") {
            (false, stripped)
        } else {
            (false, trimmed)
        };

        let (authority, path) = match without_scheme.find('/') {
            Some(idx) => (&without_scheme[..idx], &without_scheme[idx..]),
            None => (without_scheme, "/"),
        };

        if authority.is_empty() {
            return Err("Missing host in URL");
        }

        let (host, port) = match authority.find(':') {
            Some(idx) => {
                let h = &authority[..idx];
                let port_str = &authority[idx + 1..];
                let p = match parse_port_u16(port_str) {
                    Some(val) => val,
                    None => return Err("Invalid port number"),
                };
                (h, p)
            }
            None => (authority, if is_https { 443 } else { 80 }),
        };

        if host.is_empty() {
            return Err("Missing host in URL");
        }

        Ok(ParsedUrl {
            is_https,
            host,
            port,
            path,
        })
    }
}

fn parse_port_u16(s: &str) -> Option<u16> {
    if s.is_empty() {
        return None;
    }
    let mut acc = 0u32;
    for &b in s.as_bytes() {
        if !b.is_ascii_digit() {
            return None;
        }
        acc = acc.checked_mul(10)?.checked_add((b - b'0') as u32)?;
        if acc > 65535 {
            return None;
        }
    }
    Some(acc as u16)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_url_http() {
        let parsed = ParsedUrl::parse("http://example.com/api/v1").unwrap();
        assert!(!parsed.is_https);
        assert_eq!(parsed.host, "example.com");
        assert_eq!(parsed.port, 80);
        assert_eq!(parsed.path, "/api/v1");
    }

    #[test]
    fn test_parse_url_https() {
        let parsed = ParsedUrl::parse("https://keira.org/docs").unwrap();
        assert!(parsed.is_https);
        assert_eq!(parsed.host, "keira.org");
        assert_eq!(parsed.port, 443);
        assert_eq!(parsed.path, "/docs");
    }

    #[test]
    fn test_parse_url_with_port() {
        let parsed = ParsedUrl::parse("http://10.0.2.2:8080/json").unwrap();
        assert!(!parsed.is_https);
        assert_eq!(parsed.host, "10.0.2.2");
        assert_eq!(parsed.port, 8080);
        assert_eq!(parsed.path, "/json");
    }

    #[test]
    fn test_parse_url_implicit_scheme_and_path() {
        let parsed = ParsedUrl::parse("example.com").unwrap();
        assert!(!parsed.is_https);
        assert_eq!(parsed.host, "example.com");
        assert_eq!(parsed.port, 80);
        assert_eq!(parsed.path, "/");
    }

    #[test]
    fn test_parse_url_invalid() {
        assert!(ParsedUrl::parse("").is_err());
        assert!(ParsedUrl::parse("http://:80/path").is_err());
        assert!(ParsedUrl::parse("http://host:99999/path").is_err());
    }
}
