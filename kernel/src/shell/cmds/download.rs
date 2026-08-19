// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

#![allow(unused_variables, unused_unsafe)]

//!
//! Implementation of the 'download' shell command to fetch network resources over
//! encrypted HTTPS (Native TLS 1.3 Engine) or plain HTTP and save payloads to FAT16 storage.

use crate::io::vga;
use crate::net::e1000;
use crate::shell::executor::*;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    unsafe {
        if !is_admin_mode() {
            vga::set_color(vga::Color::LightRed, vga::Color::Black);
            vga::print_str(
                "Permission denied: This command requires admin privileges. Use 'please <command>'.\n",
            );
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            return;
        }

        let url = match parts.next() {
            Some("-h") | Some("--help") => {
                vga::print_str("Usage: download <URL> <target_file_path>\n\n");
                vga::print_str("Description:\n  Fetch network resources over encrypted HTTPS (Native TLS 1.3 Engine) or plain HTTP and save received payload data stream directly to FAT16 disk storage.\n\n");
                vga::print_str("Options:\n  -h, --help    Show this help message and exit\n\n");
                vga::print_str("Examples:\n  download https://google.com page.html\n");
                return;
            }
            Some(u) => u,
            None => {
                vga::set_color(vga::Color::Yellow, vga::Color::Black);
                vga::print_str("Usage: download <URL> <target_file_path>\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                return;
            }
        };

        let target_file = match parts.next() {
            Some(f) => f,
            None => {
                vga::set_color(vga::Color::Yellow, vga::Color::Black);
                vga::print_str("Usage: download <URL> <target_file_path>\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                return;
            }
        };

        e1000::init();

        let is_https = url.starts_with("https://");

        if is_https {
            // Strip https:// prefix if present
            let hostname = if url.starts_with("https://") {
                &url[8..]
            } else {
                url
            };
            let (host, path) = match hostname.find('/') {
                Some(idx) => (&hostname[..idx], &hostname[idx..]),
                None => (hostname, "/"),
            };

            vga::set_color(vga::Color::LightCyan, vga::Color::Black);
            vga::print_str("Connecting to https://");
            vga::print_str(host);
            vga::print_str(":443 (TLS 1.3 Encrypted) ...\n");
            vga::set_color(vga::Color::White, vga::Color::Black);

            match e1000::fetch_https(host, path) {
                Ok((payload, len)) => {
                    save_or_print_payload(&payload[..len], target_file, true);
                }
                Err(err) => {
                    vga::set_color(vga::Color::Yellow, vga::Color::Black);
                    vga::print_str("TLS 1.3: ");
                    vga::print_str(err);
                    vga::print_str("\nConnecting via HTTP (Port 80) ...\n");
                    vga::set_color(vga::Color::White, vga::Color::Black);

                    match e1000::fetch_http(url) {
                        Ok((payload, len)) => {
                            save_or_print_payload(&payload[..len], target_file, false);
                        }
                        Err(http_err) => {
                            vga::set_color(vga::Color::LightRed, vga::Color::Black);
                            vga::print_str("Download Error: ");
                            vga::print_str(http_err);
                            vga::print_str("\n");
                        }
                    }
                }
            }
        } else {
            vga::set_color(vga::Color::LightCyan, vga::Color::Black);
            vga::print_str("Connecting to ");
            vga::print_str(url);
            vga::print_str(" ...\n");
            vga::set_color(vga::Color::White, vga::Color::Black);

            match e1000::fetch_http(url) {
                Ok((payload, len)) => {
                    save_or_print_payload(&payload[..len], target_file, false);
                }
                Err(err) => {
                    vga::set_color(vga::Color::LightRed, vga::Color::Black);
                    vga::print_str("Error: ");
                    vga::print_str(err);
                    vga::print_str("\n");
                }
            }
        }
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    }
}

/// Helper function to save payload to FAT16 storage in the active directory path
unsafe fn save_or_print_payload(payload: &[u8], dest_path: &str, is_encrypted: bool) {
    let file_exists = if let Ok((dir_cluster, name)) = crate::fs::fat::resolve_path(dest_path) {
        crate::fs::fat::find_entry(name, dir_cluster).is_ok()
    } else {
        false
    };

    if !file_exists {
        let _ = crate::fs::fat::create_file(dest_path);
    }

    match crate::fs::fat::write_file_content(dest_path, payload) {
        Ok(_) => {
            crate::fs::fat::clear_cache();
            vga::set_color(vga::Color::LightGreen, vga::Color::Black);
            vga::print_str("Saved network payload to ");
            vga::print_str(dest_path);
            vga::print_str(" in current directory (FAT16 Disk Storage)\n");
        }
        Err(e) => {
            vga::set_color(vga::Color::LightRed, vga::Color::Black);
            vga::print_str("Error writing to ");
            vga::print_str(dest_path);
            vga::print_str(": ");
            vga::print_str(e);
            vga::print_str("\n");
            if let Ok(s) = core::str::from_utf8(payload) {
                vga::set_color(vga::Color::White, vga::Color::Black);
                vga::print_str(s);
            }
        }
    }

    vga::set_color(vga::Color::LightGreen, vga::Color::Black);
    if is_encrypted {
        vga::print_str("\n[TLS 1.3 Encrypted Download Complete: ");
    } else {
        vga::print_str("\n[Download Complete: ");
    }
    vga::print_u64(payload.len() as u64);
    vga::print_str(" bytes received]\n");
}
