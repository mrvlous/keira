// SPDX-License-Identifier: GPL-2.0-only

//! IPv4 address parsing and dotted-decimal conversion utilities.

/// Parse a dotted-decimal IPv4 string (e.g. "10.0.2.2").
pub fn parse_ipv4_addr(s: &str) -> Option<[u8; 4]> {
    let mut parts = s.split('.');
    let a = parts.next()?.parse::<u8>().ok()?;
    let b = parts.next()?.parse::<u8>().ok()?;
    let c = parts.next()?.parse::<u8>().ok()?;
    let d = parts.next()?.parse::<u8>().ok()?;
    if parts.next().is_none() {
        Some([a, b, c, d])
    } else {
        None
    }
}
