// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Implementation of the 'https' shell command to perform encrypted HTTPS GET
//! requests over the Native TLS 1.3 Cryptographic Engine.

use super::progress::print_status_badge;
use super::url::ParsedUrl;
use keira_io::vga;

/// Execute the 'https' command to inspect TLS 1.3 parameters, run SHA-256 self-test, or perform HTTPS request.
pub fn run(parts: &mut core::str::SplitWhitespace) {
    unsafe {
        let sub = parts.next();

        match sub {
            Some("-h") | Some("--help") => {
                vga::set_color(vga::Color::White, vga::Color::Black);
                vga::print_str("Usage: https <url|info|sha256>\n\n");
                vga::print_str("Description:\n  Perform encrypted HTTPS GET request over Native TLS 1.3 Engine (AES-128-GCM, X25519 ECDH, HKDF-SHA256).\n\n");
                vga::print_str("Options:\n  -h, --help    Show this help message and exit\n\n");
                vga::print_str("Subcommands:\n  info    Query Native TLS 1.3 cryptographic engine parameters and status\n  sha256  Execute FIPS 180-4 SHA-256 digest self-test\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            }
            Some("info") | None => {
                vga::set_color(vga::Color::White, vga::Color::Black);
                vga::print_str("KEIRA NATIVE TLS 1.3 ENGINE\n");
                vga::set_color(vga::Color::White, vga::Color::Black);
                vga::print_str("  Protocol    : TLS 1.3 (RFC 8446)\n");
                vga::print_str("  Cipher      : TLS_AES_128_GCM_SHA256 (0x1301)\n");
                vga::print_str("  Key Exchange: X25519 (Curve25519 ECDH)\n");
                vga::print_str("  Hash        : SHA-256 (FIPS 180-4)\n");
                vga::print_str("  AEAD        : AES-128-GCM (NIST SP 800-38D)\n");
                vga::print_str("  HMAC        : HMAC-SHA-256 (RFC 2104)\n");
                vga::print_str("  KDF         : HKDF-Expand-Label (RFC 8446 Sec 7.1)\n");
                vga::print_str("  Status      : ");
                vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                vga::print_str("Active (Kernel Native)\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            }
            Some("sha256") => {
                // Demo: compute SHA-256 of test string
                let test = b"Keira Kernel";
                let hash = keira_crypto::sha256::sha256(test);
                vga::set_color(vga::Color::White, vga::Color::Black);
                vga::print_str("SHA-256 DIGEST TEST\n");
                vga::set_color(vga::Color::White, vga::Color::Black);
                vga::print_str("  Input  : \"Keira Kernel\"\n");
                vga::print_str("  Digest : ");
                for byte in hash.iter() {
                    print_hex_byte(*byte);
                }
                vga::print_str("\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            }
            Some(url) => {
                let parsed = match ParsedUrl::parse(url) {
                    Ok(p) => p,
                    Err(err) => {
                        print_status_badge("Error", vga::Color::LightRed);
                        vga::print_str("URL parse error: ");
                        vga::print_str(err);
                        vga::print_str("\n");
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                        return;
                    }
                };

                keira_net::driver::e1000::init();

                print_status_badge("Connecting", vga::Color::LightGreen);
                vga::print_str("https://");
                vga::print_str(parsed.host);
                if parsed.port != 443 {
                    vga::print_str(":");
                    vga::print_u64(parsed.port as u64);
                }
                vga::print_str(" (TLS 1.3 AES-128-GCM, X25519 ECDH)...\n");

                match keira_net::tls::fetch_https(parsed.host, parsed.path) {
                    Ok((resp_buf, bytes)) => {
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                        if let Ok(s) = core::str::from_utf8(&resp_buf[..bytes]) {
                            vga::print_str(s);
                            if !s.ends_with('\n') {
                                vga::print_str("\n");
                            }
                        }
                        print_status_badge("Finished", vga::Color::LightGreen);
                        vga::print_str("TLS 1.3 encrypted transfer completed (");
                        vga::print_u64(bytes as u64);
                        vga::print_str(" bytes)\n");
                    }
                    Err(err) => {
                        print_status_badge("Error", vga::Color::LightRed);
                        vga::print_str("TLS 1.3 request failed: ");
                        vga::print_str(err);
                        vga::print_str("\n");
                    }
                }
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            }
        }
    }
}

/// Print a single byte as two hex digits
fn print_hex_byte(b: u8) {
    let hi = b >> 4;
    let lo = b & 0x0F;
    let hex_char = |n: u8| -> u8 {
        if n < 10 {
            b'0' + n
        } else {
            b'a' + n - 10
        }
    };
    let s = [hex_char(hi), hex_char(lo)];
    if let Ok(hex_str) = core::str::from_utf8(&s) {
        vga::print_str(hex_str);
    }
}
