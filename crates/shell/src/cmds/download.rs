// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Implementation of the 'download' shell command to stream network resources over
//! encrypted HTTPS (Native TLS 1.3 Engine) or plain HTTP and save payloads directly to FAT16 storage.

use crate::executor::*;
use keira_io::vga;
use keira_net::tcp::fetch_http_stream;
use keira_net::tls::fetch_https_stream;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    if !unsafe { is_admin_mode() } {
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
            vga::print_str("Description:\n  Stream network resources over encrypted HTTPS (Native TLS 1.3 Engine) or plain HTTP and save received payload data stream directly to FAT16 disk storage.\n\n");
            vga::print_str("Options:\n  -h, --help    Show this help message and exit\n\n");
            vga::print_str("Examples:\n  download https://example.com/app.elf /apps/app.elf\n  download http://208.95.112.1/json ip.json\n");
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

    let on_progress = |received: usize, total_opt: Option<usize>| {
        vga::set_color(vga::Color::LightCyan, vga::Color::Black);
        if let Some(total) = total_opt {
            if total > 0 {
                let percent = (received * 100) / total;
                vga::print_str("Downloading [");
                let filled = (percent * 20) / 100;
                for _ in 0..filled {
                    vga::print_str("=");
                }
                if filled < 20 {
                    vga::print_str(">");
                    for _ in filled + 1..20 {
                        vga::print_str(" ");
                    }
                }
                vga::print_str("] ");
                vga::print_u64(percent as u64);
                vga::print_str("% (");
                vga::print_u64(received as u64);
                vga::print_str("/");
                vga::print_u64(total as u64);
                vga::print_str(" bytes)\n");
            }
        }
        vga::set_color(vga::Color::White, vga::Color::Black);
    };

    if is_https {
        vga::set_color(vga::Color::LightCyan, vga::Color::Black);
        vga::print_str("Connecting to https://");
        vga::print_str(host);
        vga::print_str(":443 (TLS 1.3 Encrypted Stream) ...\n");
        vga::set_color(vga::Color::White, vga::Color::Black);

        match unsafe { fetch_https_stream(host, path, on_progress) } {
            Ok((payload, _cl)) => unsafe {
                save_or_print_payload(payload, target_file, true);
            },
            Err(err) => {
                vga::set_color(vga::Color::Yellow, vga::Color::Black);
                vga::print_str("TLS 1.3: ");
                vga::print_str(err);
                vga::print_str("\nConnecting via HTTP Stream (Port 80) ...\n");
                vga::set_color(vga::Color::White, vga::Color::Black);

                match unsafe { fetch_http_stream(raw_url, on_progress) } {
                    Ok((payload, _cl)) => unsafe {
                        save_or_print_payload(payload, target_file, false);
                    },
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
        vga::print_str("Connecting to http://");
        vga::print_str(host);
        vga::print_str(":80 (HTTP Stream) ...\n");
        vga::set_color(vga::Color::White, vga::Color::Black);

        match unsafe { fetch_http_stream(raw_url, on_progress) } {
            Ok((payload, _cl)) => unsafe {
                save_or_print_payload(payload, target_file, false);
            },
            Err(err) => {
                vga::set_color(vga::Color::LightRed, vga::Color::Black);
                vga::print_str("Download Error: ");
                vga::print_str(err);
                vga::print_str("\n");
            }
        }
    }
    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
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
            vga::set_color(vga::Color::LightGreen, vga::Color::Black);
            vga::print_str("Saved network stream to ");
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
        vga::print_str("\n[TLS 1.3 Encrypted Stream Download Complete: ");
    } else {
        vga::print_str("\n[Stream Download Complete: ");
    }
    vga::print_u64(payload.len() as u64);
    vga::print_str(" bytes received]\n");
}
