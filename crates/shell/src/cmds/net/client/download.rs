// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Implementation of the 'download' shell command to stream network resources over
//! encrypted HTTPS (Native TLS 1.3 Engine) or plain HTTP and save payloads directly to FAT16 storage.

use super::progress::{print_status_badge, ProgressRenderer};
use super::url::ParsedUrl;
use crate::args::CliArgs;
use keira_io::vga;
use keira_net::tcp::fetch_http_stream;
use keira_net::tls::fetch_https_stream;

/// Execute the 'download' command to stream a network file directly to storage.
pub fn run(parts: &mut core::str::SplitWhitespace) {
    let args = CliArgs::parse(parts);

    if args.has_flag('h', "help") || args.is_empty() {
        vga::set_color(vga::Color::White, vga::Color::Black);
        vga::print_str("Usage: download <URL> <target_file_path>\n\n");
        vga::print_str("Description:\n  Stream network resources over encrypted TLS 1.3 HTTPS or HTTP and save directly to FAT16 disk storage.\n\n");
        vga::print_str("Options:\n");
        vga::print_str("  -h, --help    Show this help message and exit\n\n");
        vga::print_str("Examples:\n");
        vga::print_str("  download https://example.com/app.elf /apps/app.elf\n");
        vga::print_str("  download http://10.0.2.2/data.bin /data/data.bin\n");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        return;
    }

    let url = match args.first_positional() {
        Some(u) => u,
        None => {
            vga::set_color(vga::Color::Yellow, vga::Color::Black);
            vga::print_str("Usage: download <URL> <target_file_path>\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            return;
        }
    };

    let target_file = match args.second_positional() {
        Some(f) => f,
        None => {
            vga::set_color(vga::Color::Yellow, vga::Color::Black);
            vga::print_str("Usage: download <URL> <target_file_path>\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            return;
        }
    };

    let parsed = match ParsedUrl::parse(url) {
        Ok(p) => p,
        Err(err) => unsafe {
            print_status_badge("Error", vga::Color::LightRed);
            vga::print_str("URL parse error: ");
            vga::print_str(err);
            vga::print_str("\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            return;
        },
    };

    unsafe {
        keira_net::driver::e1000::init();
    }

    let mut renderer = ProgressRenderer::new(28);

    unsafe {
        if parsed.is_https {
            print_status_badge("Connecting", vga::Color::LightGreen);
            vga::print_str("https://");
            vga::print_str(parsed.host);
            if parsed.port != 443 {
                vga::print_str(":");
                vga::print_u64(parsed.port as u64);
            }
            vga::print_str(" (TLS 1.3 Encrypted Stream)...\n");

            let mut on_progress = |received: usize, total_opt: Option<usize>| {
                renderer.update(received, total_opt);
            };

            match fetch_https_stream(parsed.host, parsed.path, &mut on_progress) {
                Ok((payload, _cl)) => {
                    save_payload(payload, target_file, true);
                }
                Err(err) => {
                    print_status_badge("Warning", vga::Color::Yellow);
                    vga::print_str("TLS 1.3: ");
                    vga::print_str(err);
                    vga::print_str(" -> Fallback to HTTP (Port 80)\n");

                    match fetch_http_stream(url, &mut on_progress) {
                        Ok((payload, _cl)) => {
                            save_payload(payload, target_file, false);
                        }
                        Err(http_err) => {
                            print_status_badge("Error", vga::Color::LightRed);
                            vga::print_str("Download failed: ");
                            vga::print_str(http_err);
                            vga::print_str("\n");
                        }
                    }
                }
            }
        } else {
            print_status_badge("Connecting", vga::Color::LightGreen);
            vga::print_str("http://");
            vga::print_str(parsed.host);
            if parsed.port != 80 {
                vga::print_str(":");
                vga::print_u64(parsed.port as u64);
            }
            vga::print_str(" (HTTP Stream)...\n");

            let on_progress = |received: usize, total_opt: Option<usize>| {
                renderer.update(received, total_opt);
            };

            match fetch_http_stream(url, on_progress) {
                Ok((payload, _cl)) => {
                    save_payload(payload, target_file, false);
                }
                Err(err) => {
                    print_status_badge("Error", vga::Color::LightRed);
                    vga::print_str("Download failed: ");
                    vga::print_str(err);
                    vga::print_str("\n");
                }
            }
        }

        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    }
}

/// Helper function to save payload to storage via the Virtual File System (VFS).
///
/// # Safety
/// Writes to the VGA text mode console.
unsafe fn save_payload(payload: &[u8], dest_path: &str, is_encrypted: bool) {
    if !keira_fs::vfs::exists(dest_path) {
        if let Err(e) = keira_fs::vfs::create_file(dest_path) {
            print_status_badge("Error", vga::Color::LightRed);
            vga::print_str("Failed to create '");
            vga::print_str(dest_path);
            vga::print_str("': ");
            vga::print_str(e);
            vga::print_str("\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            return;
        }
    }

    match keira_fs::vfs::write_file(dest_path, payload) {
        Ok(_) => {
            print_status_badge("Downloaded", vga::Color::LightGreen);
            vga::print_u64(payload.len() as u64);
            vga::print_str(" bytes payload (");
            if is_encrypted {
                vga::print_str("TLS 1.3 Encrypted Stream");
            } else {
                vga::print_str("HTTP Plain Stream");
            }
            vga::print_str(")\n");

            print_status_badge("Finished", vga::Color::LightGreen);
            vga::print_str("Target file '");
            vga::print_str(dest_path);
            vga::print_str("' written to storage\n");
        }
        Err(e) => {
            print_status_badge("Error", vga::Color::LightRed);
            vga::print_str("Error writing to '");
            vga::print_str(dest_path);
            vga::print_str("': ");
            vga::print_str(e);
            vga::print_str("\n");
        }
    }
    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
}
