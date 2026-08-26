// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

#![allow(unused_variables, unused_unsafe)]

//! Implementation of the 'download' shell command to stream network resources over
//! encrypted HTTPS (Native TLS 1.3 Engine) or plain HTTP and save payloads directly to FAT16 storage.

use crate::args::CliArgs;
use crate::executor::*;
use keira_io::vga;
use keira_net::tcp::fetch_http_stream;
use keira_net::tls::fetch_https_stream;

unsafe fn print_cargo_tag(tag: &str, color: vga::Color) {
    vga::set_color(color, vga::Color::Black);
    for _ in 0..(12usize.saturating_sub(tag.len())) {
        vga::print_str(" ");
    }
    vga::print_str(tag);
    vga::print_str(" ");
    vga::set_color(vga::Color::White, vga::Color::Black);
}

pub fn run(parts: &mut core::str::SplitWhitespace) {
    if !unsafe { is_admin_mode() } {
        unsafe {
            vga::set_color(vga::Color::LightRed, vga::Color::Black);
            vga::print_str(
                "Permission denied: This command requires admin privileges. Use 'please <command>'.\n",
            );
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        }
        return;
    }

    let args = CliArgs::parse(parts);

    if args.has_flag('h', "help") || args.is_empty() {
        unsafe {
            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("Usage: download <URL> <target_file_path>\n\n");
            vga::print_str("Description:\n  Stream network resources over encrypted TLS 1.3 HTTPS or HTTP and save directly to FAT16 disk storage.\n\n");
            vga::print_str("Options:\n");
            vga::print_str("  -h, --help    Show this help message and exit\n\n");
            vga::print_str("Examples:\n");
            vga::print_str("  download https://example.com/app.elf /apps/app.elf\n");
            vga::print_str("  download http://208.95.112.1/json ip.json\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        }
        return;
    }

    let url = match args.first_positional() {
        Some(u) => u,
        None => {
            unsafe {
                vga::set_color(vga::Color::Yellow, vga::Color::Black);
                vga::print_str("Usage: download <URL> <target_file_path>\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            }
            return;
        }
    };

    let target_file = match args.second_positional() {
        Some(f) => f,
        None => {
            unsafe {
                vga::set_color(vga::Color::Yellow, vga::Color::Black);
                vga::print_str("Usage: download <URL> <target_file_path>\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            }
            return;
        }
    };

    unsafe {
        keira_net::driver::e1000::init();
    }

    let is_https = url.starts_with("https://");

    let raw_url = if let Some(stripped) = url.strip_prefix("https://") {
        stripped
    } else if let Some(stripped) = url.strip_prefix("http://") {
        stripped
    } else {
        url
    };

    let (host, path) = match raw_url.find('/') {
        Some(idx) => (&raw_url[..idx], &raw_url[idx..]),
        None => (raw_url, "/"),
    };

    let on_progress = |received: usize, total_opt: Option<usize>| unsafe {
        if let Some(total) = total_opt {
            if total > 0 {
                let percent = (received * 100) / total;
                print_cargo_tag("Downloading", vga::Color::LightGreen);
                vga::print_str("[");
                let bar_width = 28usize;
                let filled = (percent * bar_width) / 100;
                for _ in 0..filled {
                    vga::print_str("=");
                }
                if filled < bar_width {
                    vga::print_str(">");
                    for _ in (filled + 1)..bar_width {
                        vga::print_str(" ");
                    }
                }
                vga::print_str("] ");
                if percent < 10 {
                    vga::print_str("  ");
                } else if percent < 100 {
                    vga::print_str(" ");
                }
                vga::print_u64(percent as u64);
                vga::print_str("% (");

                if total >= 1024 * 1024 {
                    vga::print_u64((received / (1024 * 1024)) as u64);
                    vga::print_str(".");
                    vga::print_u64(((received % (1024 * 1024)) * 10 / (1024 * 1024)) as u64);
                    vga::print_str(" MiB / ");
                    vga::print_u64((total / (1024 * 1024)) as u64);
                    vga::print_str(".");
                    vga::print_u64(((total % (1024 * 1024)) * 10 / (1024 * 1024)) as u64);
                    vga::print_str(" MiB)");
                } else if total >= 1024 {
                    vga::print_u64((received / 1024) as u64);
                    vga::print_str(".");
                    vga::print_u64(((received % 1024) * 10 / 1024) as u64);
                    vga::print_str(" KiB / ");
                    vga::print_u64((total / 1024) as u64);
                    vga::print_str(".");
                    vga::print_u64(((total % 1024) * 10 / 1024) as u64);
                    vga::print_str(" KiB)");
                } else {
                    vga::print_u64(received as u64);
                    vga::print_str(" B / ");
                    vga::print_u64(total as u64);
                    vga::print_str(" B)");
                }
                vga::print_str("\n");
            }
        }
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    };

    unsafe {
        if is_https {
            print_cargo_tag("Connecting", vga::Color::LightGreen);
            vga::print_str("https://");
            vga::print_str(host);
            if !host.contains(':') {
                vga::print_str(":443");
            }
            vga::print_str(" (TLS 1.3 Encrypted Stream)...\n");

            match fetch_https_stream(host, path, on_progress) {
                Ok((payload, _cl)) => {
                    save_or_print_payload(payload, target_file, true);
                }
                Err(err) => {
                    print_cargo_tag("Warning", vga::Color::Yellow);
                    vga::print_str("TLS 1.3: ");
                    vga::print_str(err);
                    vga::print_str(" -> Fallback to HTTP (Port 80)\n");

                    match fetch_http_stream(raw_url, on_progress) {
                        Ok((payload, _cl)) => {
                            save_or_print_payload(payload, target_file, false);
                        }
                        Err(http_err) => {
                            print_cargo_tag("error", vga::Color::LightRed);
                            vga::print_str("Download failed: ");
                            vga::print_str(http_err);
                            vga::print_str("\n");
                        }
                    }
                }
            }
        } else {
            print_cargo_tag("Connecting", vga::Color::LightGreen);
            vga::print_str("http://");
            vga::print_str(host);
            if !host.contains(':') {
                vga::print_str(":80");
            }
            vga::print_str(" (HTTP Stream)...\n");

            match fetch_http_stream(raw_url, on_progress) {
                Ok((payload, _cl)) => {
                    save_or_print_payload(payload, target_file, false);
                }
                Err(err) => {
                    print_cargo_tag("error", vga::Color::LightRed);
                    vga::print_str("Download failed: ");
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
    let file_exists = if let Ok((dir_cluster, name)) = keira_fs::fat::resolve_path(dest_path) {
        keira_fs::fat::find_entry(name, dir_cluster).is_ok()
    } else {
        false
    };

    if !file_exists {
        let _ = keira_fs::fat::create_file(dest_path);
    }

    match keira_fs::fat::write_file_content(dest_path, payload) {
        Ok(_) => {
            keira_fs::fat::clear_cache();
            print_cargo_tag("Downloaded", vga::Color::LightGreen);
            vga::print_u64(payload.len() as u64);
            vga::print_str(" bytes payload (");
            if is_encrypted {
                vga::print_str("TLS 1.3 Encrypted Stream");
            } else {
                vga::print_str("HTTP Plain Stream");
            }
            vga::print_str(")\n");

            print_cargo_tag("Finished", vga::Color::LightGreen);
            vga::print_str("target file '");
            vga::print_str(dest_path);
            vga::print_str("' written to FAT16 disk storage\n");
        }
        Err(e) => {
            print_cargo_tag("error", vga::Color::LightRed);
            vga::print_str("Error writing to '");
            vga::print_str(dest_path);
            vga::print_str("': ");
            vga::print_str(e);
            vga::print_str("\n");
            if let Ok(s) = core::str::from_utf8(payload) {
                vga::set_color(vga::Color::White, vga::Color::Black);
                vga::print_str(s);
                vga::print_str("\n");
            }
        }
    }
    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
}
